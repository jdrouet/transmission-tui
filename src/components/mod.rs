use std::sync::LazyLock;

use transmission_rpc::types::TorrentStatus;

pub(crate) mod confirm;
pub(crate) mod list;

pub(crate) const SIZE_FORMATTER: LazyLock<human_number::Formatter<'static>> =
    LazyLock::new(|| human_number::Formatter::si().with_unit("B"));
pub(crate) const SPEED_FORMATTER: LazyLock<human_number::Formatter<'static>> =
    LazyLock::new(|| human_number::Formatter::si().with_unit("B/s"));

pub(crate) fn torrent_status_label(status: TorrentStatus) -> &'static str {
    match status {
        TorrentStatus::Stopped => "Stopped",
        TorrentStatus::QueuedToVerify => "Queued to verify",
        TorrentStatus::Verifying => "Verifying",
        TorrentStatus::QueuedToDownload => "Queued to download",
        TorrentStatus::Downloading => "Downloading",
        TorrentStatus::QueuedToSeed => "Queued to seed",
        TorrentStatus::Seeding => "Seeding",
    }
}
