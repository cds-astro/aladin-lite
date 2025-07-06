mod freq_space;
mod space;

pub use freq_space::FreqSpaceMoc;
pub use space::SpaceMoc;

pub enum Moc {
    FreqSpace(FreqSpaceMoc),
    Space(SpaceMoc),
}
