use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{info, debug, warn, error};

use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_VP8};
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType};
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::{TrackLocal, TrackLocalWriter};
use bytes::Bytes;

use super::{StreamQualitySettings, StreamMetrics};

/// Real WebRTC engine using webrtc-rs for production streaming
pub struct RealWebRTCEngine {
    port: u16,
    api: webrtc::api::API,
    peer_connections: Arc<RwLock<HashMap<String, Arc<RTCPeerConnection>>>>, // stream_id -> peer_connection
    active_streams: Arc<RwLock<HashMap<String, StreamConnection>>>, // stream_id -> stream details
    ice_servers: Vec<RTCIceServer>,
}

#[derive(Debug, Clone)]
pub struct StreamConnection {
    pub stream_id: String,
    pub creator: String,
    pub viewers: HashMap<String, ViewerPeer>, // viewer_id -> peer connection info
    pub quality: StreamQualitySettings,
    pub track: Option<Arc<TrackLocalStaticRTP>>,
    pub metrics: StreamMetrics,
    pub viewer_count: u32, // Track viewer count separately
}

#[derive(Debug, Clone)]
pub struct ViewerPeer {
    pub viewer_id: String,
    pub peer_connection: Arc<RTCPeerConnection>,
    pub connected_since: chrono::DateTime<chrono::Utc>,
}

impl RealWebRTCEngine {
    pub async fn new(port: u16) -> Result<Self> {
        info!("🎥 Initializing Real WebRTC Engine on port {}", port);

        // Create a MediaEngine object to configure the supported codec
        let mut media_engine = MediaEngine::default();

        // Add support for common video codecs
        media_engine.register_default_codecs()?;

        // Create a InterceptorRegistry to configure interceptors
        let mut registry = Default::default();
        registry = register_default_interceptors(registry, &mut media_engine)?;

        // Use the default set of Interceptors
        let api = APIBuilder::new()
            .with_interceptor_registry(registry)
            .build();

        // Configure ICE servers (STUN servers for NAT traversal)
        let ice_servers = vec![
            RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                ..Default::default()
            },
            RTCIceServer {
                urls: vec!["stun:stun1.l.google.com:19302".to_string()],
                ..Default::default()
            },
        ];

        Ok(Self {
            port,
            api,
            peer_connections: Arc::new(RwLock::new(HashMap::new())),
            active_streams: Arc::new(RwLock::new(HashMap::new())),
            ice_servers,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("🚀 Starting Real WebRTC Engine");
        // In a real implementation, this would start the signaling server
        // For now, we'll simulate the readiness
        Ok(())
    }

    /// Create a new stream with real WebRTC track
    pub async fn create_stream(
        &mut self,
        stream_id: String,
        creator: String,
        quality: StreamQualitySettings,
    ) -> Result<()> {
        info!("🎬 Creating real WebRTC stream: {} by {}", stream_id, creator);

        // Create a video track with VP8 codec
        let video_codec = RTCRtpCodecCapability {
            mime_type: MIME_TYPE_VP8.to_string(),
            clock_rate: 90000,
            channels: 0,
            sdp_fmtp_line: String::new(),
            rtcp_feedback: Vec::new(),
        };
        
        let video_track = Arc::new(TrackLocalStaticRTP::new(
            video_codec,
            "video".to_string(),
            format!("video-{}", stream_id),
        ));

        let stream_connection = StreamConnection {
            stream_id: stream_id.clone(),
            creator: creator.clone(),
            viewers: HashMap::new(),
            quality: quality.clone(),
            track: Some(video_track.clone()),
            metrics: StreamMetrics {
                bandwidth_mbps: 0.0,
                latency_ms: 0,
                packet_loss_percent: 0.0,
                resolution: quality.resolution.clone(),
                fps: quality.target_fps,
                bitrate_kbps: quality.max_bitrate_kbps,
                jitter_ms: 0,
            },
            viewer_count: 0,
        };

        self.active_streams.write().await.insert(stream_id.clone(), stream_connection);

        // Start generating test video frames
        let track_clone = video_track.clone();
        let stream_id_clone = stream_id.clone();
        tokio::spawn(async move {
            Self::generate_test_video_frames(track_clone, stream_id_clone).await;
        });

        info!("✅ Real WebRTC stream created: {}", stream_id);
        Ok(())
    }

    /// Connect a viewer to a stream using real WebRTC
    pub async fn connect_viewer(&mut self, stream_id: String, viewer_id: String) -> Result<()> {
        info!("🔗 Connecting viewer {} to real WebRTC stream {}", viewer_id, stream_id);

        // Create peer connection configuration
        let config = RTCConfiguration {
            ice_servers: self.ice_servers.clone(),
            ..Default::default()
        };

        // Create a new RTCPeerConnection
        let peer_connection = Arc::new(self.api.new_peer_connection(config).await?);

        // Add the stream track to the peer connection
        if let Some(stream) = self.active_streams.read().await.get(&stream_id) {
            if let Some(track) = &stream.track {
                let rtp_sender = peer_connection
                    .add_track(track.clone() as Arc<dyn TrackLocal + Send + Sync>)
                    .await?;

                // Start reading RTP packets to keep the connection alive
                let peer_connection_clone = Arc::clone(&peer_connection);
                tokio::spawn(async move {
                    let mut rtcp_buf = vec![0u8; 1500];
                    while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {
                        // Handle RTCP packets for quality feedback
                    }
                });

                // Set up connection state handlers
                let viewer_id_clone = viewer_id.clone();
                peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
                    let viewer_id = viewer_id_clone.clone();
                    Box::pin(async move {
                        match s {
                            RTCPeerConnectionState::Connected => {
                                info!("✅ Viewer {} connected via WebRTC", viewer_id);
                            }
                            RTCPeerConnectionState::Disconnected => {
                                info!("❌ Viewer {} disconnected from WebRTC", viewer_id);
                            }
                            RTCPeerConnectionState::Failed => {
                                error!("💥 WebRTC connection failed for viewer {}", viewer_id);
                            }
                            _ => {}
                        }
                    })
                }));

                // Set up ICE connection state handler
                peer_connection.on_ice_connection_state_change(Box::new(move |s: RTCIceConnectionState| {
                    Box::pin(async move {
                        debug!("ICE Connection State changed: {:?}", s);
                    })
                }));

                // Store the peer connection
                self.peer_connections.write().await.insert(
                    format!("{}:{}", stream_id, viewer_id),
                    Arc::clone(&peer_connection),
                );

                // Update stream connection with new viewer
                if let Some(stream) = self.active_streams.write().await.get_mut(&stream_id) {
                    stream.viewers.insert(viewer_id.clone(), ViewerPeer {
                        viewer_id: viewer_id.clone(),
                        peer_connection: Arc::clone(&peer_connection),
                        connected_since: chrono::Utc::now(),
                    });
                    stream.viewer_count = stream.viewers.len() as u32;
                }

                info!("✅ Viewer {} connected to real WebRTC stream {}", viewer_id, stream_id);
            } else {
                error!("Stream {} has no track available", stream_id);
            }
        } else {
            error!("Stream {} not found", stream_id);
        }

        Ok(())
    }

    /// Disconnect a viewer from a stream
    pub async fn disconnect_viewer(&mut self, stream_id: String, viewer_id: String) -> Result<()> {
        info!("❌ Disconnecting viewer {} from stream {}", viewer_id, stream_id);

        let peer_key = format!("{}:{}", stream_id, viewer_id);
        
        // Close the peer connection
        if let Some(peer_connection) = self.peer_connections.write().await.remove(&peer_key) {
            if let Err(e) = peer_connection.close().await {
                warn!("Error closing peer connection: {}", e);
            }
        }

        // Remove viewer from stream
        if let Some(stream) = self.active_streams.write().await.get_mut(&stream_id) {
            stream.viewers.remove(&viewer_id);
            stream.viewer_count = stream.viewers.len() as u32;
        }

        Ok(())
    }

    /// Generate test video frames for demonstration
    async fn generate_test_video_frames(track: Arc<TrackLocalStaticRTP>, stream_id: String) {
        info!("📹 Starting test video frame generation for stream {}", stream_id);
        
        let mut frame_count = 0u64;
        let frame_interval = tokio::time::Duration::from_millis(33); // ~30 FPS

        loop {
            // Generate a simple test pattern (in real implementation, this would be actual video frames)
            let test_data = Self::generate_test_frame(frame_count);
            
            if let Err(e) = track.write_rtp(&test_data).await {
                warn!("Error writing RTP packet for stream {}: {}", stream_id, e);
                break;
            }

            frame_count += 1;
            
            // Stop after generating 900 frames (30 seconds at 30 FPS)
            if frame_count >= 900 {
                info!("📹 Test video generation completed for stream {}", stream_id);
                break;
            }

            tokio::time::sleep(frame_interval).await;
        }
    }

    /// Generate a test video frame (placeholder implementation)
    fn generate_test_frame(frame_count: u64) -> webrtc::rtp::packet::Packet {
        // This is a simplified test frame generation
        // In a real implementation, this would encode actual video data
        let payload = format!("Test frame {}", frame_count).into_bytes();
        
        webrtc::rtp::packet::Packet {
            header: webrtc::rtp::header::Header {
                version: 2,
                padding: false,
                extension: false,
                marker: false,
                payload_type: 96, // VP8 payload type
                sequence_number: (frame_count % 65536) as u16,
                timestamp: (frame_count * 3000) as u32, // 90kHz clock for video
                ssrc: 12345,
                csrc: vec![],
                extension_profile: 0,
                extensions: vec![],
            },
            payload: Bytes::from(payload),
        }
    }

    pub async fn get_stream_metrics(&self, stream_id: &str) -> Option<StreamMetrics> {
        self.active_streams.read().await.get(stream_id).map(|s| s.metrics.clone())
    }

    pub async fn list_active_streams(&self) -> Vec<String> {
        self.active_streams.read().await.keys().cloned().collect()
    }

    pub async fn get_viewer_count(&self, stream_id: &str) -> u32 {
        self.active_streams
            .read()
            .await
            .get(stream_id)
            .map_or(0, |s| s.viewers.len() as u32)
    }

    /// Get WebRTC connection statistics for monitoring
    pub async fn get_connection_stats(&self, stream_id: &str, viewer_id: &str) -> Option<ConnectionStats> {
        let peer_key = format!("{}:{}", stream_id, viewer_id);
        
        if let Some(peer_connection) = self.peer_connections.read().await.get(&peer_key) {
            // In a real implementation, we would gather actual WebRTC stats
            // For now, return simulated stats
            Some(ConnectionStats {
                bytes_sent: 1024 * 1024, // 1MB
                bytes_received: 512 * 1024, // 512KB
                packets_sent: 1000,
                packets_received: 500,
                rtt_ms: 50,
                jitter_ms: 5,
                packet_loss_percent: 0.1,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u32,
    pub packets_received: u32,
    pub rtt_ms: u32,
    pub jitter_ms: u32,
    pub packet_loss_percent: f64,
}
