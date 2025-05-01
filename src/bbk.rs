use std::{
    io,
    num::{ParseFloatError, ParseIntError},
    path::PathBuf,
    process::Command,
    str::FromStr,
    string::FromUtf8Error,
};

use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct BbkOutput {
    pub download: f64,
    pub upload: f64,
    pub ping: f64,
    pub server: String,
    pub isp: String,
    pub support_id: String,
    pub measurement_id: u64,
}

#[derive(Debug, Error)]
pub enum BbkError {
    #[error("failed to parse to float")]
    FloatParseError(#[from] ParseFloatError),
    #[error("failed to parse {0} to integer")]
    IntParseError(#[from] ParseIntError),
    #[error("failed to convert bbk output to string")]
    StringParseError(#[from] FromUtf8Error),
    #[error("failed to run bbk")]
    ProgramError(#[from] io::Error),
}

impl FromStr for BbkOutput {
    type Err = BbkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [
            download,
            upload,
            ping,
            server,
            isp,
            support_id,
            measurement_id,
        ] = <[&str; 7]>::try_from(s.split(",").collect::<Vec<_>>()).unwrap();

        Ok(BbkOutput {
            download: download.trim().parse()?,
            upload: upload.trim().parse()?,
            ping: ping.trim().parse()?,
            server: server.into(),
            isp: isp.into(),
            support_id: support_id.into(),
            measurement_id: measurement_id.trim().parse()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Bbk {
   pub path: PathBuf,
   pub args: Vec<String>,
}

impl Bbk {
    pub fn run_bbk(&self) -> Result<BbkOutput, BbkError> {
        let default_args = ["--csv", "--speedlimit=1", "--duration=1"].map(|s| s.to_owned());
        let args = Vec::from_iter(self.args.iter().cloned().chain(default_args));

        println!("Running BBK with arguments: {:?}", args);
        let output = Command::new(self.path.clone()).args(args).output()?;
        let result = String::from_utf8(output.stdout)?;

        let bbk_output = result.parse::<BbkOutput>();
        println!("BBK results: {:?}", bbk_output);
        bbk_output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_output_correctly() {
        let bbk_cli_output = "250.445,254.074,4.47409,anycast-global-ipv4.bredbandskollen.se,ISP AB,support_id,11111111";
        let parsed_result = bbk_cli_output.parse::<BbkOutput>().unwrap();
        let expected = BbkOutput {
            download: 250.445,
            upload: 254.074,
            ping: 4.47409,
            server: "anycast-global-ipv4.bredbandskollen.se".to_string(),
            isp: "ISP AB".to_string(),
            support_id: "support_id".to_string(),
            measurement_id: 11111111,
        };
        assert_eq!(parsed_result, expected);
    }
    #[test]
    fn it_generates_error_from_invalid_input() {
        let bbk_cli_output = "not a float,254.074,4.47409,anycast-global-ipv4.bredbandskollen.se,ISP AB,support_id,11111111";
        let parsed_result = bbk_cli_output.parse::<BbkOutput>();
        assert!(parsed_result.is_err());
        assert!(parsed_result.is_err_and(|e| match e {
            BbkError::FloatParseError(_) => true,
            _ => false,
        }));
    }
}
