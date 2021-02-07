pub mod bluetooth;
pub mod message;
pub mod serializable;

use bluetooth::{AsyncBtStream, Manager};
use message::*;
use message::data_mdr;
use message::data_mdr::nc_asm;
use serializable::Serializable;

use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, AsyncBufReadExt};

#[tokio::main]
async fn main() {
    let manager = Manager::new().unwrap();
    let devices = manager.get_devices().unwrap();
    println!("{:#?}", devices);

    let mut bt_stream = AsyncBtStream::new(devices[0].bt_stream().unwrap()).unwrap();

    println!("Connected!");

    let msg = Message {
        sequence_number: 0,
        data: Data::DataMdr(data_mdr::DataMdr {
            command: data_mdr::Command::NcAsmSetParam(nc_asm::NcAsmSetParam {
                nc_asm_inquired_type: nc_asm::NcAsmInquiredType::NoiseCancellingAndAmbientSoundMode,
                nc_asm_effect: nc_asm::NcAsmEffect::On,
                nc_asm_setting_type: nc_asm::NcAsmSettingType::DualSingleOff,
                nc_dual_single_value: nc_asm::NcDualSingleValue::Dual,
                asm_setting_type: nc_asm::AsmSettingType::LevelAdjustment,
                asm_id: nc_asm::AsmId::Normal,
                asm_level: 0,
            }),
        }),
    };

    println!("send: {:?}", msg);
    bt_stream.write_all(&msg.serialize()).await.unwrap();

    let mut buf_reader = BufReader::new(bt_stream);
    loop {
        //TODO the compiler does not optimize out initializing this memory...
        // Maybe something to think about later?
        let mut buf = [0; 2048];
        buf_reader.read(&mut buf).await.unwrap();
        println!("recv: {:?}", Message::deserialize(&mut buf).unwrap());
    }
}
