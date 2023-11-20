use std::net::UdpSocket;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionManager
};
/*
issues to fix:
close the udp socket
I think the .clone() is causing memmory issues

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

async fn get_current_session() -> Result<Option<GlobalSystemMediaTransportControlsSession>, windows::core::Error> {
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.await?;
    let session: GlobalSystemMediaTransportControlsSession = manager.GetCurrentSession()?;

    if session.SourceAppUserModelId()?.to_string().to_lowercase().contains("spotify.exe") {
        return Ok(Some(session));
    }

    return Ok(None);
}

async fn get_playing_details(session: GlobalSystemMediaTransportControlsSession) -> Result<MusicDetails, windows::core::Error> {
    let session_details = session.TryGetMediaPropertiesAsync()?.await?;

    let currently_playing_details = MusicDetails {
        song_name: session_details.Title()?.to_string(),
        song_artist: session_details.Artist()?.to_string(),
        song_is_playing: matches!(session.GetPlaybackInfo()?.PlaybackStatus()?, windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
    };

    Ok(currently_playing_details)
}

fn send_music_details_to_vrc(music_details: &MusicDetails) -> std::io::Result<()>{
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

    let udp_socket = UdpSocket::bind("0.0.0.0:0")?;
    udp_socket.send_to(formatted_message.as_bytes(), ("127.0.0.1", 9000));

    Ok(())
}

fn are_structs_same(struct1:&MusicDetails, struct2:&MusicDetails) -> bool {
    return struct1.song_name == struct2.song_name && struct1.song_artist == struct2.song_artist
}

#[tokio::main]
async fn main() {
    let mut last_music_details: MusicDetails = MusicDetails::none();

    loop {
        let h = get_current_session().await.unwrap();
        let music_details: MusicDetails = get_playing_details(h.unwrap()).await.unwrap();
        if !are_structs_same(&music_details, &last_music_details) {
            dbg!(&music_details);
            send_music_details_to_vrc(&music_details);
            last_music_details = music_details.clone();
        }
    }
}


/*
122
*/
//https://learn.microsoft.com/en-us/uwp/api/windows.media.systemmediatransportcontrols?view=winrt-22621
//https://learn.microsoft.com/en-us/uwp/api/windows.media.systemmediatransportcontrols.displayupdater?view=winrt-22621#windows-media-systemmediatransportcontrols-displayupdater
//https://learn.microsoft.com/en-us/uwp/api/windows.media.systemmediatransportcontrolsdisplayupdater?view=winrt-22621
