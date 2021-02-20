pub mod bluetooth;
pub mod message;
pub mod message_queue;
pub mod repl;
pub mod serializable;

use bluetooth::{AsyncBtStream, Manager};
use message::*;
use message::data_mdr;
use message::data_mdr::nc_asm;
use message_queue::MessageQueue;

#[tokio::main]
async fn main() {
    let mut repl = repl::Repl::new().unwrap();
    repl.run().await.unwrap();

    let manager = Manager::new().unwrap();
    let devices = manager.get_devices().unwrap();
    println!("{:#?}", devices);

    let bt_stream = AsyncBtStream::new(devices[0].bt_stream().unwrap()).unwrap();

    println!("Connected!");

    let queue = MessageQueue::new(bt_stream);
    let (mut recv_queue, mut send_queue) = queue.split();

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

    send_queue.send(msg).await.unwrap();

    loop {
        recv_queue.recv().await;
    }
}