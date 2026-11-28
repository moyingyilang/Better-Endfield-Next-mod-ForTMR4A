//! `be-bem` CLI：业务链的宿主侧入口，用于导入前的包检查与 CI 回归。
//!
//! ```text
//! be-bem inspect <file.bem> [--json]
//! be-bem validate <file.bem>
//! be-bem repack <in.bem> <out.bem> [--zstd]     # 读→写（默认 raw，--zstd 有收益才压缩）
//! be-bem pack <manifest.json> <payload-dir> <out.bem> [--minor N]
//! ```
//!
//! 退出码：0 通过，1 包不合法，2 用法/IO，3 版本不支持。

use std::process::ExitCode;

use be_bem::BemPackage;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("用法: be-bem <inspect|validate> <file.bem> [--json]");
        return ExitCode::from(2);
    }
    let command = args[0].as_str();
    let path = match args.iter().skip(1).find(|a| !a.starts_with("--")) {
        Some(path) => path,
        None => {
            eprintln!("缺少 BEM 路径");
            return ExitCode::from(2);
        }
    };
    let json = args.iter().any(|a| a == "--json");

    if command == "repack" {
        let out = match args.get(2) {
            Some(out) => out,
            None => {
                eprintln!("用法: be-bem repack <in.bem> <out.bem>");
                return ExitCode::from(2);
            }
        };
        let package = match be_bem::BemPackage::open(path) {
            Ok(package) => package,
            Err(cause) => {
                eprintln!("打开失败: {cause}");
                return ExitCode::from(cause.code() as u8);
            }
        };
        let compress = args.iter().any(|a| a == "--zstd");
        let level = args
            .windows(2)
            .find(|w| w[0] == "--level")
            .and_then(|w| w[1].parse::<i32>().ok())
            .unwrap_or(3);
        let options = be_bem::WriteOptions {
            minor: package.minor,
            compress,
            level,
        };
        return match package.repack(options).and_then(|bytes| {
            std::fs::write(out, bytes).map_err(be_bem::Error::Io)
        }) {
            Ok(()) => {
                eprintln!("repack 完成: {out}");
                ExitCode::SUCCESS
            }
            Err(cause) => {
                eprintln!("repack 失败: {cause}");
                ExitCode::from(cause.code() as u8)
            }
        };
    }

    if command == "pack" {
        let manifest_path = match args.get(1) {
            Some(path) => path,
            None => {
                eprintln!("用法: be-bem pack <manifest.json> <payload-dir> <out.bem> [--minor N]");
                return ExitCode::from(2);
            }
        };
        let payload_dir = match args.get(2) {
            Some(dir) => dir,
            None => {
                eprintln!("缺少 payload 目录");
                return ExitCode::from(2);
            }
        };
        let out = match args.get(3) {
            Some(out) => out,
            None => {
                eprintln!("缺少输出路径");
                return ExitCode::from(2);
            }
        };
        let minor = args
            .windows(2)
            .find(|w| w[0] == "--minor")
            .and_then(|w| w[1].parse::<u16>().ok())
            .unwrap_or(0);
        let manifest = match std::fs::read(manifest_path) {
            Ok(bytes) => bytes,
            Err(cause) => {
                eprintln!("读取 manifest 失败: {cause}");
                return ExitCode::from(2);
            }
        };
        let mut files: Vec<_> = match std::fs::read_dir(payload_dir) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .filter(|path| path.extension().map(|e| e == "bin").unwrap_or(false))
                .collect(),
            Err(cause) => {
                eprintln!("读取 payload 目录失败: {cause}");
                return ExitCode::from(2);
            }
        };
        files.sort();
        let mut payloads = Vec::with_capacity(files.len());
        for file in &files {
            match std::fs::read(file) {
                Ok(bytes) => payloads.push(bytes),
                Err(cause) => {
                    eprintln!("读取 payload 失败 {}: {cause}", file.display());
                    return ExitCode::from(2);
                }
            }
        }
        let compress = args.iter().any(|a| a == "--zstd");
        let options = be_bem::WriteOptions {
            minor,
            compress,
            level: 3,
        };
        return match be_bem::write_bem(options, &manifest, &payloads)
            .and_then(|bytes| std::fs::write(out, bytes).map_err(be_bem::Error::Io))
        {
            Ok(()) => {
                eprintln!("pack 完成: {out}（{} 个 payload）", payloads.len());
                ExitCode::SUCCESS
            }
            Err(cause) => {
                eprintln!("pack 失败: {cause}");
                ExitCode::from(cause.code() as u8)
            }
        };
    }

    let package = match BemPackage::open(path) {
        Ok(package) => package,
        Err(cause) => {
            eprintln!("打开失败: {cause}");
            return ExitCode::from(cause.code() as u8);
        }
    };
    let report = match package.validate() {
        Ok(report) => report,
        Err(cause) => {
            eprintln!("校验失败: {cause}");
            return ExitCode::from(cause.code() as u8);
        }
    };

    match command {
        "validate" => {
            eprintln!(
                "OK  v{}  {}  payloads={}  decoded={} 字节",
                report.format_version, report.package_id, report.payload_count, report.decoded_bytes
            );
            ExitCode::SUCCESS
        }
        "inspect" => {
            if json {
                match serde_json::to_string_pretty(&report) {
                    Ok(text) => println!("{text}"),
                    Err(cause) => {
                        eprintln!("JSON 序列化失败: {cause}");
                        return ExitCode::from(1);
                    }
                }
            } else {
                println!("format      : BEM v{}", report.format_version);
                println!("package_id  : {}", report.package_id);
                println!("name        : {}", report.name);
                println!("payloads    : {}", report.payload_count);
                println!("stored      : {} 字节", report.stored_bytes);
                println!("decoded     : {} 字节", report.decoded_bytes);
                println!("referenced  : {}", report.referenced_payloads);
                println!("meshes      : {}", report.meshes);
                println!("textures    : {}", report.textures);
                for warning in &report.warnings {
                    println!("warning     : {warning}");
                }
            }
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("未知子命令: {other}");
            ExitCode::from(2)
        }
    }
}
