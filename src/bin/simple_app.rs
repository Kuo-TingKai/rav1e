// 最簡單的 rav1e 應用程式
// 編碼一個簡單的測試影片並輸出為 IVF 格式

use rav1e::config::SpeedSettings;
use rav1e::*;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 開始編碼測試影片...");
    
    // 設定編碼器配置
    let enc = EncoderConfig {
        width: 320,           // 影片寬度
        height: 240,          // 影片高度
        speed_settings: SpeedSettings::from_preset(9), // 速度設定 (0-10, 9為較快)
        quantizer: 100,       // 量化參數 (0-255, 越高品質越差但檔案越小)
        ..Default::default()
    };

    let cfg = Config::new().with_encoder_config(enc.clone());
    let mut ctx: Context<u16> = cfg.new_context()?;

    // 創建輸出檔案
    let mut output_file = File::create("simple_output.ivf")?;
    
    // 寫入 IVF 檔案頭
    write_ivf_header(&mut output_file, enc.width, enc.height, 30)?; // 30fps

    // 創建一個簡單的測試影格 (灰色背景)
    let mut frame = ctx.new_frame();
    let gray_value = 128; // 灰色值
    let pixels = vec![gray_value; enc.width * enc.height];

    // 填充影格資料
    for p in &mut frame.planes {
        let stride = (enc.width + p.cfg.xdec) >> p.cfg.xdec;
        p.copy_from_raw_u8(&pixels, stride, 1);
    }

    let frame_count = 30; // 編碼 30 個影格
    let mut encoded_frames = 0;

    println!("📹 編碼 {} 個影格...", frame_count);

    // 發送影格進行編碼
    for i in 0..frame_count {
        match ctx.send_frame(frame.clone()) {
            Ok(_) => {
                println!("  發送影格 {}/{}", i + 1, frame_count);
            }
            Err(EncoderStatus::EnoughData) => {
                println!("  編碼器佇列已滿，無法發送影格 {}", i + 1);
                break;
            }
            Err(e) => {
                return Err(format!("發送影格失敗: {:?}", e).into());
            }
        }
    }

    // 完成編碼
    ctx.flush();

    // 接收編碼後的封包
    loop {
        match ctx.receive_packet() {
            Ok(pkt) => {
                // 寫入 IVF 影格資料
                write_ivf_frame(&mut output_file, &pkt.data, pkt.input_frameno)?;
                encoded_frames += 1;
                println!("  ✅ 編碼完成影格 {} (大小: {} bytes)", 
                        pkt.input_frameno, pkt.data.len());
            }
            Err(EncoderStatus::LimitReached) => {
                println!("🎬 編碼完成！總共編碼了 {} 個影格", encoded_frames);
                break;
            }
            Err(EncoderStatus::NeedMoreData) => {
                // 需要更多資料，但我們已經發送完所有影格
                break;
            }
            Err(EncoderStatus::Encoded) => {
                // 編碼完成，繼續接收下一個封包
                continue;
            }
            Err(e) => {
                return Err(format!("接收封包失敗: {:?}", e).into());
            }
        }
    }

    println!("💾 輸出檔案: simple_output.ivf");
    println!("🎉 編碼完成！");
    
    Ok(())
}

// 寫入 IVF 檔案頭
fn write_ivf_header(file: &mut File, width: usize, height: usize, fps: u32) -> std::io::Result<()> {
    // IVF 檔案頭格式
    file.write_all(b"DKIF")?;           // 簽名
    file.write_all(&0u16.to_le_bytes())?; // 版本
    file.write_all(&32u16.to_le_bytes())?; // 檔案頭大小
    file.write_all(b"AV01")?;           // 編碼格式 (AV1)
    file.write_all(&(width as u16).to_le_bytes())?;  // 寬度
    file.write_all(&(height as u16).to_le_bytes())?; // 高度
    file.write_all(&fps.to_le_bytes())?; // 幀率
    file.write_all(&1u32.to_le_bytes())?; // 時間刻度
    file.write_all(&0u32.to_le_bytes())?; // 影格數量 (未知)
    file.write_all(&0u32.to_le_bytes())?; // 保留
    Ok(())
}

// 寫入 IVF 影格資料
fn write_ivf_frame(file: &mut File, data: &[u8], frame_number: u64) -> std::io::Result<()> {
    // 影格頭
    file.write_all(&(data.len() as u32).to_le_bytes())?; // 影格大小
    file.write_all(&frame_number.to_le_bytes())?;        // 時間戳
    file.write_all(data)?;                               // 影格資料
    Ok(())
} 