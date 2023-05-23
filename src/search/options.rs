#[derive(Debug, Clone, Copy)]
pub struct SearchOptions {
    pub depth: Option<u8>,
    pub movetime: Option<u32>,
    pub infinite: bool,
    pub wtime: Option<u32>,
    pub btime: Option<u32>,
    pub winc: Option<u32>,
    pub binc: Option<u32>,
    pub movestogo: Option<u32>,
}

impl SearchOptions {
    pub fn new() -> SearchOptions {
        return SearchOptions {
            depth: None,
            // movetime: Some(3000),
            movetime: None,
            infinite: false,
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            movestogo: None,
        };
    }
}

impl From<Vec<&str>> for SearchOptions {
    fn from(tokens: Vec<&str>) -> SearchOptions {
        let mut options = SearchOptions::new();

        for i in 1..tokens.len() {
            match tokens[i] {
                "infinite" => options.infinite = true,
                "depth" | "binc" | "winc" | "btime" | "wtime" | "movestogo" | "movetime" => {
                    let value = Some(tokens[i + 1].parse::<u32>().unwrap());
                    match tokens[i] {
                        "movetime" => options.movetime = value,
                        "depth" => options.depth = Some(value.unwrap() as u8),
                        "binc" => options.binc = value,
                        "winc" => options.winc = value,
                        "btime" => options.btime = value,
                        "wtime" => options.wtime = value,
                        "movestogo" => options.movestogo = value,
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        return options;
    }
}
