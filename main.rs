use std::net::UdpSocket;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionManager
};

/*
TODO:
get_playing_details doesnt have error handling
add 5 second cool down
*/

#[derive(Debug, Clone)]
struct MusicDetails {
    song_name: String,
    song_artist: String,
    song_is_playing: bool
}

impl MusicDetails {
    fn none() -> Self {
        Self {
            song_name: String::new(),
            song_artist: String::new(),
            song_is_playing: false
        }
    }
}

async fn get_current_session() -> GlobalSystemMediaTransportControlsSession {
    loop {
        let manager = match GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
            Ok(manager) => {
                match manager.await {
                    Ok(manager) => manager,
                    Err(e) => {
                        eprintln!("1: {}", e);
                        continue;
                    }
                }
            }
            Err(e) => {
                eprintln!("2: {}", e);
                continue;
            }
        };

        match manager.GetCurrentSession() {
            Ok(session) => {
                if session.SourceAppUserModelId().unwrap().to_string().to_lowercase().contains("spotify.exe") {
                    return session
                }
            }
            Err(e) => {
                //eprintln!("Cant get session: {}", e);
                continue;
            }
        } 
    }
}

async fn get_playing_details(session: &GlobalSystemMediaTransportControlsSession) -> Result<MusicDetails, windows::core::Error> {
    let session_details = session.TryGetMediaPropertiesAsync()?.await?;

    let currently_playing_details = MusicDetails {
        song_name: session_details.Title()?.to_string(),
        song_artist: session_details.Artist()?.to_string(),
        song_is_playing: matches!(session.GetPlaybackInfo()?.PlaybackStatus()?, windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
    };

    Ok(currently_playing_details)
}

fn send_music_details_to_vrc(udp_socket: &UdpSocket, music_details: &MusicDetails) -> std::io::Result<()>{
    let path = "/chatbox/input";
    let separator = [0x00, 0x00, 0x2c, 0x73, 0x54, 0x46, 0x00, 0x00, 0x00, 0x00];

    let message = if music_details.song_is_playing {
        format!("{song_name} - {song_artist}", song_name=music_details.song_name, song_artist=music_details.song_artist)
    } else {
        "Paused!".to_string()
    };

    let separatorb: &[u8] = &separator;

    let mut formatted_message = format!("{path}{separator}{message}", path=path, separator=String::from_utf8_lossy(separatorb), message=message);

    let endingb: &[u8] = &[0x00, 0x00, 0x00];
    formatted_message.push_str(&String::from_utf8_lossy(endingb));

    
    udp_socket.send_to(formatted_message.as_bytes(), ("127.0.0.1", 9000));

    Ok(())
}

fn are_structs_same(struct1:&MusicDetails, struct2:&MusicDetails) -> bool {
    return struct1.song_name == struct2.song_name && struct1.song_artist == struct2.song_artist && struct1.song_is_playing == struct2.song_is_playing
}

#[tokio::main]
async fn main() {
    let udp_socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let mut session = get_current_session().await;
    let mut last_music_details: MusicDetails = MusicDetails::none();

    loop {
        let music_details = match get_playing_details(&session).await {
            Ok(details) => details,
            Err(e) => {
                eprintln!("Waiting for spotify; Error getting music details: {:?}", e);
                session = get_current_session().await;
                continue;
            }
        };

        if !are_structs_same(&music_details, &last_music_details) {
            send_music_details_to_vrc(&udp_socket, &music_details);
            last_music_details = music_details.clone();
        }
    }
}

/*
https://learn.microsoft.com/en-us/uwp/api/windows.media.systemmediatransportcontrols?view=winrt-22621
https://learn.microsoft.com/en-us/uwp/api/windows.media.systemmediatransportcontrols.displayupdater?view=winrt-22621#windows-media-systemmediatransportcontrols-displayupdater
https://learn.microsoft.com/en-us/uwp/api/windows.media.systemmediatransportcontrolsdisplayupdater?view=winrt-22621
*/
