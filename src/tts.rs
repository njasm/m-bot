use std::io::Read;
use std::rc::Rc;

pub type SpeechResponse = Box<dyn Read + Send + Sync>;

pub trait TextToSpeech {
    fn get_speech(&mut self, text: &str) -> Result<SpeechResponse, &'static str>;
}

pub struct VoiceRSS {
    url: String,
}

impl Default for VoiceRSS {
    fn default() -> Self {
        let key = std::env::var("VOICERSS_TOKEN").expect("VOICERSS TOKEN");
        let value = format!("http://api.voicerss.org/?key={}&c=wav&f=48khz_16bit_stereo&r=4&hl=en-us&b64=false&src=", key);
        Self { url: value }
    }
}

impl TextToSpeech for VoiceRSS {
    fn get_speech(&mut self, text: &str) -> Result<SpeechResponse, &'static str> {
        let url = format!("{}{}", self.url, text);
        match reqwest::blocking::get(url.as_str()) {
            Ok(r) => Ok(Box::new(r)),
            Err(_) => Err("Unable to make request"),
        }
    }
}

pub struct AzureTextToSpeech {
    issue_token_url: String,
    url: String,
    token: Option<String>,
    client: Option<reqwest::blocking::Client>,
}

impl Default for AzureTextToSpeech {
    fn default() -> Self {
        let token_url =
            std::env::var("AZURE_COGNITIVE_TOKEN_ENDPOINT").expect("AZURE TOKEN ENDPOINT");

        let url = std::env::var("AZURE_COGNITIVE_TTS_ENDPOINT").expect("AZURE TOKEN ENDPOINT");

        Self {
            issue_token_url: token_url,
            url: url,
            token: None,
            client: Some(reqwest::blocking::Client::new()),
        }
    }
}

impl AzureTextToSpeech {
    fn get_client(&mut self) -> Option<reqwest::blocking::Client> {
        if let None = self.token {
            let key = std::env::var("AZURE_COGNITIVE_KEY").expect("AZURE COGNITIVE KEY");
            let c = reqwest::blocking::Client::new();
            let token = c
                .post(self.issue_token_url.as_str())
                .header("Ocp-Apim-Subscription-Key", key)
                .header("content-length", "0")
                .send()
                .unwrap()
                .text()
                .unwrap();

            self.token = Some(token);
            info!("TOKEN: {:?}", self.token);
        };

        self.client.clone()
    }
}

impl TextToSpeech for AzureTextToSpeech {
    fn get_speech(&mut self, _text: &str) -> Result<SpeechResponse, &'static str> {
        let url = self.url.clone();
        let client = self.get_client().unwrap();
        let text = format!(
            "<speak version=\"1.0\" xmlns=\"https://www.w3.org/2001/10/synthesis\" xml:lang=\"en-US\"><voice xml:lang='en-US' name=\"en-US-Guy24kRUS\"><prosody rate=\"+20.00%\">{}</prosody></voice></speak>", 
            _text
        );

        let t = self.token.clone();
        let jwt = format!("{} {}", "Bearer", t.unwrap());
        info!("JWT HEADER: {}", jwt);

        match client
            .post(url.as_str())
            .header("Authorization", jwt)
            .header("Content-Type", "application/ssml+xml")
            .header("User-Agent", "cognitive-discord-rs")
            .header("X-Microsoft-OutputFormat", "riff-24khz-16bit-mono-pcm")
            .body(text)
            .send()
        {
            Ok(r) => {
                //let result = Ok(Box::new(r));

                extern crate sample;
                extern crate hound;

                use hound::{WavReader, WavWriter};
                use sample::{interpolate, ring_buffer, signal, Sample, Signal};
                let reader = WavReader::new(r).unwrap();

                // Get the wav spec and create a target with the new desired sample rate.
                let spec = reader.spec();
                info!("ORIGINAL SPEC: {:?}", spec);
                let mut target = spec;
                target.sample_rate = 48_000;
                info!("TARGET SPEC: {:?}", target);
                // Read the interleaved samples and convert them to a signal.
                let samples = reader
                    .into_samples()
                    .filter_map(Result::ok)
                    .map(i16::to_sample::<f64>);
                let signal = signal::from_interleaved_samples_iter(samples);

                // Convert the signal's sample rate using `Sinc` interpolation.
                let ring_buffer = ring_buffer::Fixed::from([[0.0]; 100]);
                let sinc = interpolate::Sinc::new(ring_buffer);
                let new_signal =
                    signal.from_hz_to_hz(sinc, spec.sample_rate as f64, target.sample_rate as f64);

                // Write the result to an in memory buffer that implements std::io::Read.
                let mut buffer = std::io::Cursor::new(Vec::new());
                let mut writer = WavWriter::new(&mut buffer, target).unwrap();
                for frame in new_signal.until_exhausted() {
                    writer.write_sample(frame[0].to_sample::<i16>()).unwrap();
                }

                let _ = writer.finalize();

                Ok(Box::new(buffer))
            }
            Err(_) => Err("Unable to make request"),
        }
    }
}
