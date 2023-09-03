use dds;
pub use std::cell::OnceCell;

pub const DEAL: OnceCell<DealMock> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct DealMock {
    hands: [[usize; 4]; 4],
}

impl IntoIterator for DealMock {
    type Item = [usize; 4];
    type IntoIter = std::array::IntoIter<[usize; 4], 4>;
    fn into_iter(self) -> Self::IntoIter {
        self.hands.into_iter()
    }
}

impl dds::AsDDSDeal for DealMock {
    fn as_dds_deal(&self) -> dds::DDSDealRepr {
        let mut remain_cards = [[0; 4]; 4];
        for (seat, hand) in self.clone().into_iter().enumerate() {
            for (index, suit) in hand.into_iter().enumerate() {
                remain_cards[seat][index] = (suit >> (16 * index)) as u32;
            }
        }
        dds::DDSDealRepr::new(remain_cards)
    }
}

pub struct ContractMock {}

impl dds::ContractScorer for ContractMock {
    fn score(&self, tricks: u8) -> i32 {
        0
    }
}

impl dds::AsDDSContract for ContractMock {
    fn as_dds_contract(&self) -> (i32, i32) {
        (2, 2)
    }
}

pub fn initialize_test() -> DealMock {
    DEAL.get_or_init(|| DealMock {
        hands: [
            [1580, 3145728, 71468255805440, 5215168368495034368],
            [26624, 1233649664, 171798691840, 3459890413727383552],
            [80, 608436224, 9431748182016, 364791569817010176],
            [4480, 301989888, 59648505806848, 182395784908505088],
        ],
    })
    .clone()
}
