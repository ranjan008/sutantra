use anyhow::Result;
use tokio::sync::mpsc;
use tracing::info;

use super::{StreamingCommand, StreamQualitySettings};

/// Test trigger to automatically create streams for demonstration
pub struct StreamTestTrigger {
    command_tx: mpsc::Sender<StreamingCommand>,
}

impl StreamTestTrigger {
    pub fn new(command_tx: mpsc::Sender<StreamingCommand>) -> Self {
        Self { command_tx }
    }

    /// Start automatic test stream creation after a delay
    pub async fn start_test_sequence(&self) -> Result<()> {
        info!("🧪 Starting automatic stream test sequence...");
        
        // Wait a bit for the system to stabilize
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Trigger stream creation
        info!("🎬 AUTO-TRIGGERING: Creating test stream 'demo-stream-001'");
        self.command_tx.send(StreamingCommand::PrepareStream {
            stream_id: "demo-stream-001".to_string(),
            creator: "node-creator".to_string(),
        }).await?;
        
        // Wait a moment then start the stream
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        info!("▶️ AUTO-TRIGGERING: Starting stream with VP8 codec");
        self.command_tx.send(StreamingCommand::StartStream {
            stream_id: "demo-stream-001".to_string(),
            creator: "node-creator".to_string(),
            quality_settings: StreamQualitySettings {
                video_codec: "VP8".to_string(),
                audio_codec: "Opus".to_string(),
                max_bitrate_kbps: 2000,
                target_fps: 30,
                resolution: "1280x720".to_string(),
                adaptive_bitrate: true,
            },
        }).await?;
        
        // Wait then connect a viewer
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        info!("👥 AUTO-TRIGGERING: Connecting test viewer 'demo-viewer-001'");
        self.command_tx.send(StreamingCommand::GrantAccess {
            stream_id: "demo-stream-001".to_string(),
            viewer: "demo-viewer-001".to_string(),
        }).await?;
        
        self.command_tx.send(StreamingCommand::ConnectToStream {
            stream_id: "demo-stream-001".to_string(),
            viewer: "demo-viewer-001".to_string(),
        }).await?;
        
        info!("✅ Auto-test sequence completed - streaming should now be active!");
        
        Ok(())
    }
}
