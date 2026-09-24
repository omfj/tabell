use scraper::ElementRef;
use tabled::{Table, Tabled, settings::Style};

const TABLE_URL: &str = "https://www.eliteserien.no/tabell";

#[derive(Tabled, Default)]
struct Team {
    /// The position in the league table
    #[tabled(rename = "Pos")]
    position: usize,

    /// The name of the team
    #[tabled(rename = "Team")]
    name: String,

    /// Number of games played
    #[tabled(rename = "Played")]
    played: u32,

    /// Number of wins
    #[tabled(rename = "W")]
    wins: u32,

    /// Number of draws
    #[tabled(rename = "D")]
    draws: u32,

    /// Number of losses
    #[tabled(rename = "L")]
    losses: u32,

    /// Goals scored by the team
    #[tabled(rename = "+")]
    goals_for: u32,

    /// Goals conceded by the team
    #[tabled(rename = "-")]
    goals_against: u32,

    /// Total points
    #[tabled(rename = "Points")]
    points: u32,
}

struct Eliteserien {
    url: String,
}

impl Eliteserien {
    fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }

    async fn get_table(&self) -> anyhow::Result<Vec<Team>> {
        let response = reqwest::get(self.url.clone()).await?;
        let body = response.text().await?;

        let document = scraper::Html::parse_document(&body);
        let selector = scraper::Selector::parse("table.table").unwrap();
        let table = document.select(&selector).next().ok_or_else(|| {
            anyhow::anyhow!("Failed to find the table with selector: {:?}", selector)
        })?;
        let rows_selector = scraper::Selector::parse("tr").unwrap();
        let rows = table.select(&rows_selector);

        let mut teams: Vec<Team> = Vec::new();

        for (i, row) in rows.into_iter().enumerate() {
            let cells_selector = scraper::Selector::parse("td").unwrap();
            let cells = row.select(&cells_selector);

            let mut team = Team {
                position: i,
                ..Team::default()
            };

            for (index, cell) in cells.enumerate() {
                match index {
                    1 => team.name = parse_team_name_from_cell(cell),
                    2 => team.played = parse_number_from_cell(cell),
                    3 => team.wins = parse_number_from_cell(cell),
                    4 => team.draws = parse_number_from_cell(cell),
                    5 => team.losses = parse_number_from_cell(cell),
                    6 => team.goals_for = parse_number_from_cell(cell),
                    7 => team.goals_against = parse_number_from_cell(cell),
                    9 => team.points = parse_number_from_cell(cell),
                    _ => continue,
                }
            }

            if !team.name.is_empty() {
                teams.push(team);
            }
        }

        Ok(teams)
    }
}

fn parse_number_from_cell(cell: ElementRef) -> u32 {
    cell.text()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .parse()
        .unwrap_or(0)
}

fn parse_team_name_from_cell(cell: ElementRef) -> String {
    let selector = scraper::Selector::parse("span.table__typo--full").unwrap();
    cell.select(&selector).next().map_or(String::new(), |span| {
        span.text().collect::<Vec<_>>().join(" ").trim().to_string()
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    let args = std::env::args().collect::<Vec<String>>();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("plain");
    if command == "help" || command == "--help" || command == "-h" {
        println!("Usage: {} [plain|ascii|rounded|modern]", args[0]);
        return Ok(());
    }

    let eliteserien = Eliteserien::new(TABLE_URL);
    let teams = eliteserien.get_table().await?;

    let mut table = Table::new(teams);
    match command {
        "plain" => table.with(Style::blank()),
        "ascii" => table.with(Style::ascii()),
        "rounded" => table.with(Style::ascii_rounded()),
        "modern" => table.with(Style::modern()),
        _ => table.with(Style::ascii_rounded()),
    };
    println!("{table}");

    Ok(())
}
