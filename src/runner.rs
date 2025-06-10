use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use transmission_rpc::types::{Id, TorrentGetField};

pub(crate) struct Runner {
    client: transmission_rpc::TransClient,
    action_receiver: UnboundedReceiver<crate::Action>,
    event_sender: UnboundedSender<crate::Event>,
}

impl Runner {
    pub(crate) fn new(
        client: transmission_rpc::TransClient,
        action_receiver: UnboundedReceiver<crate::Action>,
        event_sender: UnboundedSender<crate::Event>,
    ) -> Self {
        Self {
            client,
            action_receiver,
            event_sender,
        }
    }

    async fn delete_torrent(&mut self, id: i64, delete_local_data: bool) -> crate::Event {
        let _ = self.event_sender.send(crate::Event::TorrentDeleteStart(id));
        match self
            .client
            .torrent_remove(vec![Id::Id(id)], delete_local_data)
            .await
        {
            Ok(_) => crate::Event::TorrentDelete(id),
            Err(err) => crate::Event::TorrentDeleteError(id, err),
        }
    }

    async fn refresh_list(&mut self) -> crate::Event {
        let _ = self.event_sender.send(crate::Event::TorrentListUpdateStart);
        let fields = vec![
            TorrentGetField::Id,
            TorrentGetField::Error,
            TorrentGetField::ErrorString,
            TorrentGetField::Eta,
            TorrentGetField::IsFinished,
            TorrentGetField::IsStalled,
            TorrentGetField::LeftUntilDone,
            TorrentGetField::MetadataPercentComplete,
            TorrentGetField::Name,
            TorrentGetField::PeersConnected,
            TorrentGetField::PeersGettingFromUs,
            TorrentGetField::PeersSendingToUs,
            TorrentGetField::PercentDone,
            TorrentGetField::QueuePosition,
            TorrentGetField::RateDownload,
            TorrentGetField::RateUpload,
            TorrentGetField::RecheckProgress,
            TorrentGetField::SeedRatioMode,
            TorrentGetField::SeedRatioLimit,
            TorrentGetField::SizeWhenDone,
            TorrentGetField::Status,
            TorrentGetField::TotalSize,
            // TorrentGetField::Trackers,
            TorrentGetField::DownloadDir,
            TorrentGetField::UploadedEver,
            TorrentGetField::UploadRatio,
            TorrentGetField::WebseedsSendingToUs,
        ];
        match self.client.torrent_get(Some(fields), None).await {
            Ok(list) => crate::Event::TorrentListUpdate(list.arguments.torrents),
            Err(err) => crate::Event::TorrentListUpdateError(err),
        }
    }

    async fn refresh_torrent(&mut self, id: i64) -> crate::Event {
        let _ = self.event_sender.send(crate::Event::TorrentUpdateStart);
        let fields = vec![
            TorrentGetField::Id,
            TorrentGetField::Error,
            TorrentGetField::ErrorString,
            TorrentGetField::Eta,
            TorrentGetField::IsFinished,
            TorrentGetField::IsStalled,
            TorrentGetField::LeftUntilDone,
            TorrentGetField::MetadataPercentComplete,
            TorrentGetField::Name,
            TorrentGetField::PeersConnected,
            TorrentGetField::PeersGettingFromUs,
            TorrentGetField::PeersSendingToUs,
            TorrentGetField::PercentDone,
            TorrentGetField::QueuePosition,
            TorrentGetField::RateDownload,
            TorrentGetField::RateUpload,
            TorrentGetField::RecheckProgress,
            TorrentGetField::SeedRatioMode,
            TorrentGetField::SeedRatioLimit,
            TorrentGetField::SizeWhenDone,
            TorrentGetField::Status,
            TorrentGetField::TotalSize,
            // TorrentGetField::Trackers,
            TorrentGetField::DownloadDir,
            TorrentGetField::UploadedEver,
            TorrentGetField::UploadRatio,
            TorrentGetField::WebseedsSendingToUs,
        ];
        match self
            .client
            .torrent_get(Some(fields), Some(vec![Id::Id(id)]))
            .await
        {
            Ok(mut list) => {
                crate::Event::TorrentUpdate(Box::from(list.arguments.torrents.pop().unwrap()))
            }
            Err(err) => crate::Event::TorrentUpdateError(err),
        }
    }

    pub(crate) async fn run(mut self) {
        while let Some(action) = self.action_receiver.recv().await {
            let event = match action {
                crate::Action::RefreshList => self.refresh_list().await,
                crate::Action::RefreshTorrent(id) => self.refresh_torrent(id).await,
                crate::Action::DeleteTorrent(id, delete_local_data) => {
                    self.delete_torrent(id, delete_local_data).await
                }
            };
            let _ = self.event_sender.send(event);
        }
    }
}
