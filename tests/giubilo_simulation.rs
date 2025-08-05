use dds::{
    analyseplay::PlayTraceBinIter, doubledummy::DoubleDummySolver, solver::BridgeSolver,
    tables::DdTableCalculator,
};
use rand::seq::SliceRandom;
use squeezer::*;

#[test]
fn giubilo_simulation() {
    let contracts = [
        Contract::from_str("5CN", Vulnerable::No).unwrap(),
        Contract::from_str("5DN", Vulnerable::No).unwrap(),
        Contract::from_str("3NN", Vulnerable::No).unwrap(),
        Contract::from_str("4HN", Vulnerable::No).unwrap(),
        Contract::from_str("4SN", Vulnerable::No).unwrap(),
    ];
    let mut count = 0;
    let dd_solver = DoubleDummySolver::new();
    let slam_dealer = DealerBuilder::new()
        .with_function(filter_for_slam)
        .build()
        .unwrap();
    let manche_dealer = DealerBuilder::new()
        .with_function(filter_for_manche)
        .build()
        .unwrap();
    let mut deals = Vec::with_capacity(200);
    while count < 140 {
        let deal = slam_dealer.deal().unwrap();
        if contracts
            .iter()
            .map(|c| dd_solver.dd_tricks(&deal, c).unwrap())
            .any(|e| e > 11)
        {
            count += 1;
            deals.push(deal);
            println!("Deal {}", count);
        }
    }
    while count <= 200 {
        let deal = manche_dealer.deal().unwrap();
        if contracts
            .iter()
            .map(|c| (c, dd_solver.dd_tricks(&deal, c).unwrap()))
            .any(|(c, e)| {
                println!("{}", c.score(e));
                c.score(e) >= 100
            })
        {
            count += 1;
            deals.push(deal);
            println!("Deal {}", count);
        }
    }
    deals.shuffle(&mut rand::rng());
    for (i, deal) in deals.iter().enumerate() {
        println!("{}", deal.as_lin(i as u8 + 1))
    }
}

fn filter_for_slam(hands: &Hands) -> bool {
    zar_points(*hands.north()) + zar_points(*hands.south()) >= 63
}

fn filter_for_manche(hands: &Hands) -> bool {
    (zar_points(*hands.north()) + zar_points(*hands.south())) >= 53
        && (zar_points(*hands.north()) + zar_points(*hands.south())) <= 60
}
