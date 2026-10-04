#[path = "fixture_emitters.rs"]
mod emitters;
pub use emitters::{edit, save, setup};
use serde_json::{Value, json};
pub fn definition() -> Value {
    let mut def = emitters::definition();
    def["positioning"] = json!(null);
    let table = |key: &str, name: &str, mode: &str, from: u16, to: u16, value: u16| json!({"key":key,"name":name,"mode":mode,"dmxFrom":from,"dmxTo":to,"dmxDefault":value});
    let channels = def["channels"].as_array_mut().unwrap();
    for (owner, coarse) in [("pattern", 8), ("wash", 15)] {
        channels.push(
            json!({"attribute":format!("emitter.{owner}.shutter"),"coarse":coarse,"fine":null,
          "defaultValue":{"functionKey":"open","position":0},"functions":[
          table("open","开光","slot",0,15,0),table("strobe","受控频闪","range",16,255,16)]}),
        );
    }
    channels.push(
        json!({"attribute":"emitter.pattern.color-wheel","coarse":9,"fine":null,
      "defaultValue":{"functionKey":"open","position":0},"functions":[
      table("open","通光","slot",0,15,0),table("slot-one","未知色片1","slot",16,31,16)]}),
    );
    channels.push(
        json!({"attribute":"emitter.pattern.gobo-wheel","coarse":10,"fine":null,
      "defaultValue":{"functionKey":"open","position":0},"functions":[
      table("open","通光","slot",0,7,0),table("slot-one","未知图案1","slot",8,15,8),
      table("shake-one","图案1抖动","range",119,127,119)]}),
    );
    def
}
