use scraper::ElementRef;
use tabled::{settings::Style, Table, Tabled};

const TABLE_URL: &str = "https://www.eliteserien.no/tabell";

#[derive(Tabled)]
struct Team {
    #[tabled(rename = "Team")]
    name: String,
    #[tabled(rename = "Played")]
    played: u32,
    #[tabled(rename = "W")]
    wins: u32,
    #[tabled(rename = "D")]
    draws: u32,
    #[tabled(rename = "L")]
    losses: u32,
    #[tabled(rename = "+")]
    goals_for: u32,
    #[tabled(rename = "-")]
    goals_against: u32,
    #[tabled(rename = "Points")]
    points: u32,
}

fn parse_number_from_cell(cell: ElementRef) -> u32 {
    cell.text()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .parse()
        .unwrap_or(0)
}

fn parse_team_nane_from_cell(cell: ElementRef) -> String {
    let span_selector = scraper::Selector::parse("span.table__typo--full").unwrap();
    if let Some(span) = cell.select(&span_selector).next() {
        span.text().collect::<Vec<_>>().join(" ").trim().to_string()
    } else {
        String::new()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    let response = reqwest::get(TABLE_URL).await?;
    let body = response.text().await?;

    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("table.table").unwrap();
    let table = document
        .select(&selector)
        .next()
        .ok_or_else(|| anyhow::anyhow!("Failed to find the table with selector: {:?}", selector))?;
    let rows_selector = scraper::Selector::parse("tr").unwrap();
    let rows = table.select(&rows_selector);

    let mut teams: Vec<Team> = Vec::new();

    for row in rows {
        let cells_selector = scraper::Selector::parse("td").unwrap();
        let cells = row.select(&cells_selector);
        let mut team = Team {
            name: String::new(),
            played: 0,
            wins: 0,
            draws: 0,
            losses: 0,
            goals_for: 0,
            goals_against: 0,
            points: 0,
        };
        for (index, cell) in cells.enumerate() {
            match index {
                1 => team.name = parse_team_nane_from_cell(cell),
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

    let mut table = Table::new(teams);
    table.with(Style::ascii_rounded());
    println!("{table}");

    Ok(())
}
