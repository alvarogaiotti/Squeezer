use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_until},
    character::complete::{char, digit1, multispace0},
    combinator::{map_res, opt, peek},
    multi::many1,
    sequence::terminated,
};
use std::{fmt::Debug, str::FromStr};

use crate::{
    Card, Cards, Contract, Doubled, Hand, Hands, Seat, Strain, Vulnerability, Vulnerable,
    linparser2::{Bid, LinDeal, LinFile, PlaySequence, Players, Results, TournamentInfo},
};

use super::types::{LinBidding, RecordedBid, Room};

fn ws<'a, F>(mut inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, F::Output, F::Error>
where
    F: nom::Parser<&'a str>,
{
    move |input: &'a str| {
        let (input, _) = multispace0(input)?;
        let (input, result) = inner.parse(input)?;
        let (input, _) = opt(tag("pg||")).parse(input)?;
        let (input, _) = multispace0(input)?;
        Ok((input, result))
    }
}

// VG line parser: vg|tournament,round,range,start,end,team1,score1,team2,score2|
fn parse_vugraph_data(input: &str) -> IResult<&str, TournamentInfo> {
    let (input, _) = tag("vg|")(input)?;
    let (input, content) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    let parts: Vec<&str> = content.split(',').collect();
    if parts.len() >= 9 {
        Ok((
            input,
            TournamentInfo {
                tournament: parts[0].to_string(),
                round: parts[1].to_string(),
                board_range: parts[2].to_string(),
                board_start: parts[3].parse().unwrap_or(0),
                board_end: parts[4].parse().unwrap_or(0),
                team1: parts[5].to_string(),
                team1_score: parts[6].parse().unwrap_or(0),
                team2: parts[7].to_string(),
                team2_score: parts[8].parse().unwrap_or(0),
            },
        ))
    } else {
        // Fallback for malformed vg lines
        Ok((
            input,
            TournamentInfo {
                tournament: content.to_string(),
                round: String::new(),
                board_range: String::new(),
                board_start: 0,
                board_end: 0,
                team1: String::new(),
                team1_score: 0,
                team2: String::new(),
                team2_score: 0,
            },
        ))
    }
}

// RS line parser: rs|result1,result2,...|
fn parse_results(input: &str) -> IResult<&str, Results> {
    let (input, _) = ws(tag("rs|"))(input)?;
    let (input, content) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    let results = content.split(',').map(ToString::to_string).collect();
    Ok((input, Results { results }))
}

// PN line parser: pn|player1,player2,player3,player4,...|
fn parse_players(input: &str) -> IResult<&str, Players> {
    let (input, _) = ws(tag("pn|"))(input)?;
    let (input, content) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    let players = content.split(',').map(ToString::to_string).collect();
    Ok((input, Players { players }))
}

// QX line parser: qx|deal_id|
fn parse_room_and_board_number(input: &str) -> IResult<&str, (Option<Room>, u8)> {
    if let (input, Some(_)) = opt(ws(tag("qx|"))).parse(input)? {
        let (input, deal_id) = ws(take_until("|")).parse(input)?;
        let (input, _) = ws(char('|'))(input)?;

        Ok((
            input,
            (
                match deal_id.get(0..1) {
                    Some("o") => Some(Room::Open),
                    Some("c") => Some(Room::Closed),
                    _ => None,
                },
                deal_id
                    .get(1..)
                    .ok_or(nom::Err::Error(nom::error::Error::new(
                        "deal_id",
                        nom::error::ErrorKind::Char,
                    )))
                    .map(|num| num.parse::<u8>().unwrap())?,
            ),
        ))
    } else {
        // Special case to go and take board number information since LIN format suck and has not a uniform way to tag board number.
        let (input, _) = opt((tag("rh||ah|Board "), digit1, char('|'))).parse(input)?;
        let (input, (_, _, board_num)) = peek((
            take_until("ah|Board "),
            tag("ah|Board "),
            terminated(map_res(digit1, str::parse), char('|')),
        ))
        .parse(input)?;

        Ok((input, (None, board_num)))
    }
}

// ST line parser: st||
fn consume_start_table_tag(input: &str) -> IResult<&str, ()> {
    let (input, _) = ws(tag("st||"))(input)?;
    Ok((input, ()))
}

// MD line parser: md|dealer,hand1,hand2,hand3|
fn parse_dealer_and_hands(input: &str) -> IResult<&str, (Seat, Hands)> {
    let (input, _) = ws(tag("md|"))(input)?;
    let (input, content) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    let dealer: Seat = if let Some(dealer) = content.get(0..1) {
        (dealer.parse::<u8>().unwrap() + 1).into()
    } else {
        return Err(nom::Err::Error(nom::error::Error::new(
            "dealer",
            nom::error::ErrorKind::Char,
        )));
    };
    let mut deck = Cards::ALL;
    let mut hands = content
        .get(1..)
        .unwrap()
        .split(',')
        .map(|hand_str| build_hands_from_str(hand_str, &mut deck))
        .collect::<Result<Vec<Hand>, _>>()
        .map_err(|e| nom::Err::Error(nom::error::Error::new(e, nom::error::ErrorKind::Char)))?;
    if hands.len() == 3 {
        hands.push(Hand::try_from(deck).unwrap());
    }

    // Lin format has player start from south.
    hands.rotate_right(2);

    // Special case to remove board number information since the LIN format sucks
    let (input, _) = opt((tag("rh||ah|Board "), digit1, char('|'))).parse(input)?;

    Ok((input, (dealer, Hands::new_from(hands.try_into().unwrap()))))
}

fn build_hands_from_str<'a>(hand_str: &'a str, deck: &mut Cards) -> Result<Hand, &'a str> {
    let mut hand = Cards::from_str(hand_str).map_err(|_e| hand_str)?;
    *deck -= hand;
    // We should get a empty [`Hand`] only at the end of the stream.
    // Looks really error prone and should be corrected, so:
    // TODO: FIX ME
    if hand.is_empty() {
        if deck.len() == 13 {
            hand = *deck;
        } else {
            return Err(hand_str);
        }
    }
    Hand::try_from(hand).map_err(|_| hand_str)
}

// SV line parser: sv|vulnerability|
fn parse_vulnerability(input: &str) -> IResult<&str, Vulnerability> {
    let (input, _) = ws(tag("sv|"))(input)?;
    let (input, vulnerability) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    Ok((
        input,
        match vulnerability {
            "n" => Vulnerability::NS,
            "e" => Vulnerability::EW,
            "o" => Vulnerability::None,
            "b" => Vulnerability::All,
            _ => {
                return Err(nom::Err::Error(nom::error::Error::new(
                    "vuln",
                    nom::error::ErrorKind::Char,
                )));
            }
        },
    ))
}

// MB line parser: mb|bid| or mb|bid!| with optional annotation
fn parse_bid(input: &str) -> IResult<&str, RecordedBid> {
    let (input, _) = ws(tag("mb|"))(input)?;
    let (input, mut bid_str) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    // Check for annotation (indicated by !)
    if bid_str.ends_with('!') {
        bid_str = bid_str.trim_end_matches('!');
    }
    let bid = Bid::from_str(bid_str).map_err(|_| {
        nom::Err::Error(nom::error::Error::new(
            "bid_str",
            nom::error::ErrorKind::Char,
        ))
    })?;
    let (input, alert) = opt(debug_parser("Alert", parse_alert)).parse(input)?;
    Ok((input, RecordedBid { bid, alert }))
}

// AN line parser: an|annotation text|
fn parse_alert(input: &str) -> IResult<&str, String> {
    let (input, _) = ws(tag("an|"))(input)?;
    let (input, content) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    Ok((input, content.to_string()))
}

// PC line parser: pc|card|
fn parse_played_card(input: &str) -> IResult<&str, Card> {
    let (input, _) = ws(tag("pc|"))(input)?;
    let (input, card) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    Ok((
        input,
        Card::from_str(card).map_err(|_| {
            nom::Err::Error(nom::error::Error::new("card", nom::error::ErrorKind::Char))
        })?,
    ))
}

// MC line parser: mc|score|
fn parse_claim(input: &str) -> IResult<&str, u8> {
    let (input, _) = ws(tag("mc|"))(input)?;
    let (input, score) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    let score = score.parse::<u8>().unwrap_or(0);
    Ok((input, score))
}

#[cfg(debug_assertions)]
fn debug_parser<'a, F>(
    name: &'static str,
    mut parser: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, F::Output, F::Error>
where
    F: nom::Parser<&'a str>,
    F::Error: Debug,
    F::Output: Debug,
{
    move |input: &'a str| {
        println!(
            "🔍 Trying parser '{}' on: {:?}",
            name,
            &input[..input.len().min(50)]
        );
        match parser.parse(input) {
            Ok((remaining, result)) => {
                println!("✅ Parser '{name}' succeeded. Result: {result:?}");
                println!("   Remaining: {:?}", &remaining[..remaining.len().min(50)]);
                Ok((remaining, result))
            }
            Err(e) => {
                println!("❌ Parser '{name}' failed: {e:?}");
                println!("   At position: {:?}", &input[..input.len().min(20)]);
                Err(e)
            }
        }
    }
}

// Parse a single deal
fn parse_deal(input: &str) -> IResult<&str, LinDeal> {
    let (input, (room, number)) = if cfg!(debug_assertions) {
        debug_parser("Room and board", parse_room_and_board_number).parse(input)?
    } else {
        parse_room_and_board_number.parse(input)?
    };
    let (input, ()) = if cfg!(debug_assertions) {
        debug_parser("Consume start table", consume_start_table_tag).parse(input)?
    } else {
        consume_start_table_tag.parse(input)?
    };
    let (input, (dealer, hands)) = if cfg!(debug_assertions) {
        debug_parser("Dealer and hands", parse_dealer_and_hands).parse(input)?
    } else {
        parse_dealer_and_hands.parse(input)?
    };
    let (input, vulnerability) = if cfg!(debug_assertions) {
        debug_parser("SV line", parse_vulnerability)(input)?
    } else {
        parse_vulnerability(input)?
    };

    // Parse bidding sequence
    let (input, bidding) = if cfg!(debug_assertions) {
        many1(debug_parser("Bid", parse_bid)).parse(input)?
    } else {
        many1(parse_bid).parse(input)?
    };

    let bidding = LinBidding::try_from(bidding.clone()).map_err(|_| {
        nom::Err::Error(nom::error::Error::new(
            "bidding",
            nom::error::ErrorKind::Fail,
        ))
    })?;
    let contract = if let Some(contract) = compute_contract(&bidding, dealer, number) {
        Some(contract)
    } else {
        return Ok((
            input,
            LinDeal {
                room,
                contract: compute_contract(&bidding, dealer, number),
                number,
                dealer,
                hands,
                vulnerability,
                bidding,
                play: None,
                claim: None,
            },
        ));
    };

    let (input, play_sequence) = if cfg!(debug_assertions) {
        many1(terminated(
            many1(debug_parser("Played card", parse_played_card)),
            debug_parser("Pg opt matching", opt(ws(tag("pg||")))),
        ))
        .parse(input)?
    } else {
        many1(terminated(many1(parse_played_card), opt(ws(tag("pg||"))))).parse(input)?
    };

    let play_sequence = play_sequence.into_iter().flatten().collect();

    // Parse optional mc line
    let (remaining, claim) = if cfg!(debug_assertions) {
        debug_parser("Claim", opt(parse_claim)).parse(input)?
    } else {
        opt(parse_claim).parse(input)?
    };

    // Extract values from optional parsers with defaults
    //println!("Remaining input after deal parsing:\n{input}");

    Ok((
        remaining,
        LinDeal {
            room,
            contract,
            number,
            dealer,
            hands,
            vulnerability,
            bidding,
            play: Some(PlaySequence::new(play_sequence)),
            claim,
        },
    ))
}

fn compute_contract(bidding: &LinBidding, dealer: Seat, deal_number: u8) -> Option<Contract> {
    let mut declarer = None;
    let mut last_bidder = dealer as usize + bidding.len();
    let mut doubled = Doubled::NotDoubled;
    let mut strain_found = (false, Strain::NoTrumps, 7u8);
    for bid in bidding.iter().rev() {
        if strain_found.0 {
            match bid.bid {
                Bid::Contract(_, strain) if strain == strain_found.1 => {
                    last_bidder -= 1;
                    declarer = Some(last_bidder);
                }
                _ => last_bidder -= 1,
            }
        } else {
            match bid.bid {
                Bid::Double => {
                    doubled = Doubled::Doubled;
                    last_bidder -= 1;
                }
                Bid::Pass => last_bidder -= 1,
                Bid::Redouble => {
                    doubled = Doubled::Redoubled;
                    last_bidder -= 1;
                }
                Bid::Contract(level, strain) => {
                    strain_found = (true, strain, level.into());
                    last_bidder -= 1;
                    declarer = Some(last_bidder);
                }
            }
        }
    }

    if let Some(declarer) = declarer {
        let declarer = declarer.into();
        let vuln = Vulnerable::from_number_and_seat(deal_number, declarer);
        Some(Contract::new(
            strain_found.2,
            strain_found.1,
            declarer,
            vuln,
            doubled,
        ))
    } else {
        None
    }
}

// Parse the entire file
/// # Errors
/// When parsing fails
pub fn parse_bridge_file(input: &str) -> IResult<&str, LinFile> {
    let (input, _) = multispace0(input)?;
    println!("Parsing bridge file...");
    let (input, tournament_info) = opt(parse_vugraph_data).parse(input)?;
    println!("Parsed tournament info");
    let (input, results) = opt(parse_results).parse(input)?;
    println!("Parsed results");
    let (input, players) = parse_players.parse(input)?;
    println!("Parsed players");
    let (input, deals) = many1(parse_deal).parse(input)?;
    println!("Completed deal parsing");

    Ok((
        input,
        LinFile {
            tournament_info,
            results,
            players,
            deals,
        },
    ))
}

#[cfg(test)]
mod tests {

    use crate::linparser2::parser::{
        parse_bid, parse_dealer_and_hands, parse_played_card, parse_room_and_board_number,
        parse_vugraph_data,
    };
    use crate::{Card, Hand, Vulnerability, linparser2::*};
    #[test]
    fn test_parse_vg_line() {
        let input = "vg|2nd Super College Cup,Final 2,I,15,28,NCCU,0,CGU,0|";
        let (_, result) = parse_vugraph_data(input).unwrap();

        assert_eq!(result.tournament, "2nd Super College Cup");
        assert_eq!(result.round, "Final 2");
        assert_eq!(result.board_start, 15);
        assert_eq!(result.board_end, 28);
        assert_eq!(result.team1, "NCCU");
        assert_eq!(result.team1_score, 0);
    }

    #[test]
    fn test_parse_qx_line() {
        let input = "qx|o15|";
        let (_, (room, deal_number)) = parse_room_and_board_number(input).unwrap();

        assert_eq!(room, Some(Room::Open));
        assert_eq!(deal_number, 15);
    }

    #[test]
    fn test_parse_mb_line() {
        let input = "mb|3C|";
        let (_, result) = parse_bid(input).unwrap();

        assert_eq!(result.bid, Bid::from_str("3C").unwrap());

        let input_with_annotation = "mb|4C!|an|alert|";
        let (_, result) = parse_bid(input_with_annotation).unwrap();

        assert_eq!(result.bid, Bid::from_str("4C").unwrap());
        assert_eq!(result.alert, Some("alert".to_string()));
    }

    #[test]
    fn test_parse_pc_line() {
        let input = "pc|c3|";
        let (_, result) = parse_played_card(input).unwrap();

        assert_eq!(result, Card::C3);
    }

    #[test]
    fn test_parse_md_line() {
        let input = "md|1SQ8H9DT87CAKT8765,S943HJ75D962CJ942,SKJ62HKT8642DQ4C3|";
        let (_, result) = parse_dealer_and_hands(input).unwrap();

        assert_eq!(result.0, Seat::South);
        assert_eq!(result.1[2], Hand::from_str("SQ8H9DT87CAKT8765").unwrap());
    }

    #[test]
    fn test_partial_file_parse() {
        let sample_input = "vg|2nd Super College Cup,Final 2,I,15,28,NCCU,0,CGU,1|\nrs|4HW-4,4HWx-2,4HS+1,3NS-5,1NS+1,1NN+1,6HE+1,7HE=,2SS+1,2SS+2,3NW+1,3NW+1,4SE+1,4SW+1,4HW+2,4HW+2,6NS=,6NS=,3NN=,4HS=,1NN-1,1NN-1,2SS-1,3SSx-4,3CN-1,3NE+1,4HW=,3HW+2|\npn|hanhan,tim,Joshua,Thomas YCH,South,West,North,East|pg||\nqx|o15|st||md|1SQ8H9DT87CAKT8765,S943HJ7D962CQJ942,SKJ62HKT8642DQ4C3|sv|n|mb|3C|mb|p|mb|p|mb|d|mb|p|mb|p|mb|p|pc|cA|pc|c5|pc|c2|pc|h3|pg||pc|hA|pc|h9|pc|h2|pc|h8|pg||pc|hQ|pc|h7|pc|hK|pc|h6|pg||pc|dQ|pc|d2|pc|d3|pc|c4|pg||pc|dJ|pc|dK|pc|dA|pc|s4|pg||pc|dT|pc|c3|pc|d8|pc|d4|pg||pc|d6|pc|c6|pc|s8|pc|d9|pg||mc|12|pg||";

        let result = parse_bridge_file(sample_input);
        assert!(result.is_ok(), "Failed to parse sample input: {result:?}");

        let (_, bridge_file) = result.unwrap();
        assert_eq!(
            bridge_file.tournament_info.unwrap(),
            TournamentInfo {
                tournament: "2nd Super College Cup".to_string(),
                round: "Final 2".to_string(),
                board_range: "I".to_string(),
                board_start: 15,
                board_end: 28,
                team1: "NCCU".to_string(),
                team1_score: 0,
                team2: "CGU".to_string(),
                team2_score: 1
            }
        );
        assert_eq!(bridge_file.deals.len(), 1);

        let game = &bridge_file.deals[0];
        assert_eq!(game.room, Some(Room::Open));
        assert_eq!(game.number, 15);
        assert_eq!(game.dealer, Seat::South);
        assert_eq!(game.vulnerability, Vulnerability::NS);
        println!("{game:?}");
    }
    #[test]
    fn test_complete_vugraph() {
        let vugraph = include_str!("../../tests/83469.lin");
        let (_, lin) = parse_bridge_file(vugraph).unwrap();
        println!("{:?}", lin.tournament_info);
        assert_eq!(lin.deals.len(), 28);
    }
    use crate::{Contract, Doubled, Seat, Strain, Vulnerable};

    use std::str::FromStr;

    #[test]
    fn parses_lin2_test0() {
        let lin = String::from(
            "pn|simodra,fra97,matmont,thevava|st||md|3S34JH258TQKD2JQC7,S27TH69D679TKAC23,S6QH47JD458C468JA,|rh||ah|Board 1|sv|o|mb|p|mb|1S|mb|2H|mb|2S|mb|3H|mb|4S|mb|p|mb|p|mb|p|pg||pc|C7|pc|C3|pc|CA|pc|C5|pg||pc|H4|pc|HA|pc|H5|pc|H6|pg||pc|SA|pc|S3|pc|S2|pc|S6|pg||pc|SK|pc|S4|pc|S7|pc|SQ|pg||pc|D3|pc|D2|pc|DA|pc|D5|pg||pc|DK|pc|D4|pc|H3|pc|DJ|pg||pc|C2|pc|C4|pc|C9|pc|SJ|pg||pc|HK|mc|11|",
        );
        let (_, parsed_lin) = parse_bridge_file(&lin).unwrap();
        assert_eq!(
            parsed_lin.deals[0].contract.unwrap(),
            Contract::new(
                4,
                Strain::Spades,
                Seat::East,
                Vulnerable::No,
                Doubled::NotDoubled
            )
        );
    }
    #[test]
    fn parses_lin2_test1() {
        let lin = String::from(
            "pn|gattochef,sebyx,Inter2018,fede00|st||md|3SAQ432HQJT72DT3CQ,SKJH983D974CT9876,S965HK654DKJ6CAJ5,ST87HADAQ852CK432|rh||ah|Board 1|sv|o|mb|1C|an|2+|mb|1D|mb|1H|an|picche|mb|2D|mb|p|mb|p|mb|3H|mb|p|mb|3S|mb|p|mb|4S|mb|p|mb|p|mb|p|pg||pc|DA|pc|D3|pc|D9|pc|D6|pg||pc|HA|pc|H2|pc|H9|pc|H4|pg||pc|D8|pc|DT|pc|D7|pc|DJ|pg||pc|S5|pc|S7|pc|SA|pc|SJ|pg||pc|CQ|pc|CT|pc|CA|pc|C2|pg||pc|S6|pc|S8|pc|SQ|pc|SK|pg||pc|H8|mc|9|",
        );
        let (_, parsed_lin) = parse_bridge_file(&lin).unwrap();
        assert_eq!(
            parsed_lin.deals[0].contract.unwrap(),
            Contract::new(
                4,
                Strain::Spades,
                Seat::North,
                Vulnerable::No,
                Doubled::NotDoubled
            )
        );
    }
    #[test]
    fn parses_lin2_test2() {
        let lin = String::from(include_str!("../../tests/4207070707.lin"));
        let (_, parsed_lin) = parse_bridge_file(&lin).unwrap();
        assert_eq!(
            parsed_lin.deals[0].contract.unwrap(),
            Contract::new(
                4,
                Strain::Spades,
                Seat::North,
                Vulnerable::No,
                Doubled::NotDoubled
            )
        );
    }
    #[test]
    fn parses_lin2_test3() {
        let lin = String::from(include_str!("../../tests/4207076395.lin"));
        let (_, parsed_lin) = parse_bridge_file(&lin).unwrap();
        assert_eq!(
            parsed_lin.deals[0].contract.unwrap(),
            Contract::new(
                6,
                Strain::Diamonds,
                Seat::North,
                Vulnerable::Yes,
                Doubled::NotDoubled
            )
        );
    }
    #[test]
    fn parses_lin2_test4() {
        let lin = String::from(include_str!("../../tests/4207079732.lin"));
        let (_, parsed_lin) = parse_bridge_file(&lin).unwrap();
        assert_eq!(
            parsed_lin.deals[0].contract.unwrap(),
            Contract::new(
                3,
                Strain::NoTrumps,
                Seat::West,
                Vulnerable::Yes,
                Doubled::NotDoubled
            )
        );
    }
    #[test]
    fn parses_lin2_test6() {
        let lin = String::from(include_str!("../../tests/4207083254.lin"));
        let (_, parsed_lin) = parse_bridge_file(&lin).unwrap();
        assert_eq!(
            parsed_lin.deals[0].contract.unwrap(),
            Contract::new(
                3,
                Strain::Clubs,
                Seat::South,
                Vulnerable::Yes,
                Doubled::NotDoubled
            )
        );
    }
    #[test]
    fn parses_lin2_test7() {
        let lin = String::from(include_str!("../../tests/4207093356.lin"));
        let (_, _) = parse_bridge_file(&lin).unwrap();
    }
    #[test]
    fn parses_lin2_test8() {
        let lin = String::from(include_str!("../../tests/4207094241.lin"));
        let (_, parsed_lin) = parse_bridge_file(&lin).unwrap();
        assert_eq!(parsed_lin.deals[0].contract, None);
    }
}
