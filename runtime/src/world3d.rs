use std::fmt::Write as _;

#[derive(Debug, Clone, Copy, PartialEq)] pub struct Vec3 { pub x:f32,pub y:f32,pub z:f32 }
impl Vec3 { pub const fn new(x:f32,y:f32,z:f32)->Self{Self{x,y,z}} }
#[derive(Debug, Clone, Copy, PartialEq)] pub struct Transform3D { pub position:Vec3,pub rotation:Vec3,pub scale:Vec3 }
impl Transform3D { pub const fn new(position:Vec3,rotation:Vec3,scale:Vec3)->Self{Self{position,rotation,scale}} }
#[derive(Debug, Clone, PartialEq)] pub struct Camera3D { pub id:String,pub transform:Transform3D,pub fov:f32 }
#[derive(Debug, Clone, PartialEq)] pub enum Primitive3D { Box,Plane,Sphere }
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum EpistemicVisualState { Neutral,Uncertain,Evidence,Retained,Reopened,Committed }
impl EpistemicVisualState { fn as_str(self)->&'static str{match self{Self::Neutral=>"neutral",Self::Uncertain=>"uncertain",Self::Evidence=>"evidence",Self::Retained=>"retained",Self::Reopened=>"reopened",Self::Committed=>"committed"}} }
#[derive(Debug, Clone, PartialEq)] pub struct Material3D { pub color:Vec3,pub emissive:f32,pub opacity:f32,pub state:EpistemicVisualState }
impl Material3D { pub const fn new(color:Vec3,emissive:f32,opacity:f32,state:EpistemicVisualState)->Self{Self{color,emissive,opacity,state}} }
#[derive(Debug, Clone, PartialEq)] pub struct Object3D { pub id:String,pub primitive:Primitive3D,pub transform:Transform3D,pub parent:Option<String>,pub interactive:Option<String>,pub material:Material3D,pub visible:bool }
#[derive(Debug, Clone, PartialEq)] pub enum LightKind3D { Point,Directional,Ambient }
#[derive(Debug, Clone, PartialEq)] pub struct Light3D { pub id:String,pub kind:LightKind3D,pub position:Vec3,pub intensity:f32 }
#[derive(Debug, Clone, PartialEq)] pub enum WorldEvent3D { RotateY{object:String,degrees:f32,duration:f32},MoveTo{object:String,position:Vec3,duration:f32},SetVisible{object:String,visible:bool},SetEpistemicState{object:String,state:EpistemicVisualState} }
#[derive(Debug, Clone, PartialEq)] pub struct World3D { pub id:String,pub camera:Camera3D,pub objects:Vec<Object3D>,pub lights:Vec<Light3D>,pub events:Vec<WorldEvent3D> }

impl World3D {
 pub fn the_door()->Self{
  let t=|p,s|Transform3D::new(p,Vec3::new(0.0,0.0,0.0),s);
  let obj=|id:&str,p,s,parent:Option<&str>,interactive:Option<&str>,color,state|Object3D{id:id.into(),primitive:Primitive3D::Box,transform:t(p,s),parent:parent.map(Into::into),interactive:interactive.map(Into::into),material:Material3D::new(color,if state==EpistemicVisualState::Uncertain{0.22}else{0.0},1.0,state),visible:true};
  Self{id:"evacuation_corridor".into(),camera:Camera3D{id:"player".into(),transform:t(Vec3::new(0.0,1.7,5.0),Vec3::new(1.0,1.0,1.0)),fov:70.0},objects:vec![
   obj("floor",Vec3::new(0.0,-0.1,0.0),Vec3::new(8.0,0.2,18.0),None,None,Vec3::new(0.065,0.075,0.095),EpistemicVisualState::Neutral),
   obj("wall_left",Vec3::new(-3.3,1.5,-4.1),Vec3::new(4.2,3.0,0.3),None,None,Vec3::new(0.18,0.19,0.21),EpistemicVisualState::Neutral),
   obj("wall_right",Vec3::new(3.3,1.5,-4.1),Vec3::new(4.2,3.0,0.3),None,None,Vec3::new(0.18,0.19,0.21),EpistemicVisualState::Neutral),
   obj("header",Vec3::new(0.0,3.15,-4.1),Vec3::new(2.4,0.3,0.3),None,None,Vec3::new(0.18,0.19,0.21),EpistemicVisualState::Neutral),
   obj("frame_left",Vec3::new(-1.25,1.5,-3.9),Vec3::new(0.18,3.0,0.35),None,None,Vec3::new(0.36,0.37,0.39),EpistemicVisualState::Neutral),
   obj("frame_right",Vec3::new(1.25,1.5,-3.9),Vec3::new(0.18,3.0,0.35),None,None,Vec3::new(0.36,0.37,0.39),EpistemicVisualState::Neutral),
   obj("door_hinge",Vec3::new(-1.12,1.5,-3.72),Vec3::new(0.001,0.001,0.001),None,None,Vec3::new(0.2,0.2,0.2),EpistemicVisualState::Neutral),
   obj("door",Vec3::new(1.1,0.0,0.0),Vec3::new(2.2,2.8,0.16),Some("door_hinge"),Some("open"),Vec3::new(0.30,0.055,0.045),EpistemicVisualState::Uncertain),
   obj("latch",Vec3::new(2.0,0.0,0.13),Vec3::new(0.14,0.22,0.12),Some("door_hinge"),Some("latch_sensor_recently_serviced"),Vec3::new(0.55,0.32,0.06),EpistemicVisualState::Uncertain),
   obj("passage_floor",Vec3::new(0.0,-0.05,-7.0),Vec3::new(2.2,0.1,6.0),None,None,Vec3::new(0.04,0.07,0.06),EpistemicVisualState::Neutral),
   obj("stairwell",Vec3::new(0.0,1.2,-9.0),Vec3::new(1.7,2.4,0.18),None,Some("stairwell"),Vec3::new(0.08,0.28,0.20),EpistemicVisualState::Neutral),
   obj("security_camera",Vec3::new(-2.5,2.65,-2.5),Vec3::new(0.32,0.22,0.5),None,Some("camera_has_blind_spot"),Vec3::new(0.13,0.28,0.38),EpistemicVisualState::Uncertain),
   obj("reopened_marker",Vec3::new(0.0,3.45,-3.7),Vec3::new(0.8,0.07,0.07),None,None,Vec3::new(0.65,0.08,0.08),EpistemicVisualState::Reopened)
  ],lights:vec![Light3D{id:"emergency".into(),kind:LightKind3D::Point,position:Vec3::new(0.0,2.7,1.0),intensity:0.8},Light3D{id:"ambient".into(),kind:LightKind3D::Ambient,position:Vec3::new(0.0,0.0,0.0),intensity:0.18}],events:Vec::new()}.with_hidden("reopened_marker")
 }
 fn with_hidden(mut self,id:&str)->Self{if let Some(o)=self.objects.iter_mut().find(|o|o.id==id){o.visible=false;}self}
 fn set_state(&mut self,id:&str,state:EpistemicVisualState){if let Some(o)=self.objects.iter_mut().find(|o|o.id==id){o.material.state=state;o.material.emissive=match state{EpistemicVisualState::Uncertain=>0.22,EpistemicVisualState::Evidence=>0.55,EpistemicVisualState::Retained=>0.3,EpistemicVisualState::Reopened=>0.8,EpistemicVisualState::Committed=>0.15,EpistemicVisualState::Neutral=>0.0};}self.events.push(WorldEvent3D::SetEpistemicState{object:id.into(),state});}
 pub fn apply_action(&mut self,action:&str,reopened:bool){match action{"latch_sensor_recently_serviced"=>self.set_state("latch",EpistemicVisualState::Evidence),"camera_has_blind_spot"=>self.set_state("security_camera",EpistemicVisualState::Evidence),"open"=>{self.events.push(WorldEvent3D::RotateY{object:"door_hinge".into(),degrees:-92.0,duration:1.0});self.set_state("door",EpistemicVisualState::Retained);self.events.push(WorldEvent3D::MoveTo{object:"player".into(),position:Vec3::new(0.0,1.7,-5.3),duration:1.8});},"stairwell"=>{self.events.push(WorldEvent3D::MoveTo{object:"player".into(),position:Vec3::new(0.0,1.7,-8.0),duration:1.5});self.set_state("stairwell",EpistemicVisualState::Committed)},"retreat"=>{self.events.push(WorldEvent3D::MoveTo{object:"player".into(),position:Vec3::new(0.0,1.7,7.0),duration:1.2});self.set_state("door",EpistemicVisualState::Committed)},_=>{}}if reopened{self.set_state("door",EpistemicVisualState::Reopened);if let Some(o)=self.objects.iter_mut().find(|o|o.id=="reopened_marker"){o.visible=true;}self.events.push(WorldEvent3D::SetVisible{object:"reopened_marker".into(),visible:true});}}
 pub fn to_json(&self)->String{format!("{{\"schema\":3,\"id\":{},\"camera\":{},\"objects\":[{}],\"lights\":[{}],\"events\":[{}]}}",json_string(&self.id),camera_json(&self.camera),self.objects.iter().map(object_json).collect::<Vec<_>>().join(","),self.lights.iter().map(light_json).collect::<Vec<_>>().join(","),self.events.iter().map(event_json).collect::<Vec<_>>().join(","))}
}
fn vec3_json(v:Vec3)->String{format!("[{:?},{:?},{:?}]",v.x,v.y,v.z)}
fn transform_json(v:Transform3D)->String{format!("{{\"position\":{},\"rotation\":{},\"scale\":{}}}",vec3_json(v.position),vec3_json(v.rotation),vec3_json(v.scale))}
fn primitive_name(v:&Primitive3D)->&'static str{match v{Primitive3D::Box=>"box",Primitive3D::Plane=>"plane",Primitive3D::Sphere=>"sphere"}}
fn camera_json(v:&Camera3D)->String{format!("{{\"id\":{},\"transform\":{},\"fov\":{:?}}}",json_string(&v.id),transform_json(v.transform),v.fov)}
fn material_json(v:&Material3D)->String{format!("{{\"color\":{},\"emissive\":{:?},\"opacity\":{:?},\"state\":{}}}",vec3_json(v.color),v.emissive,v.opacity,json_string(v.state.as_str()))}
fn object_json(v:&Object3D)->String{format!("{{\"id\":{},\"primitive\":{},\"transform\":{},\"parent\":{},\"interactive\":{},\"material\":{},\"visible\":{}}}",json_string(&v.id),json_string(primitive_name(&v.primitive)),transform_json(v.transform),v.parent.as_ref().map(|x|json_string(x)).unwrap_or_else(||"null".into()),v.interactive.as_ref().map(|x|json_string(x)).unwrap_or_else(||"null".into()),material_json(&v.material),v.visible)}
fn light_json(v:&Light3D)->String{let kind=match v.kind{LightKind3D::Point=>"point",LightKind3D::Directional=>"directional",LightKind3D::Ambient=>"ambient"};format!("{{\"id\":{},\"kind\":{},\"position\":{},\"intensity\":{:?}}}",json_string(&v.id),json_string(kind),vec3_json(v.position),v.intensity)}
fn event_json(v:&WorldEvent3D)->String{match v{WorldEvent3D::RotateY{object,degrees,duration}=>format!("{{\"kind\":\"rotate_y\",\"object\":{},\"degrees\":{:?},\"duration\":{:?}}}",json_string(object),degrees,duration),WorldEvent3D::MoveTo{object,position,duration}=>format!("{{\"kind\":\"move_to\",\"object\":{},\"position\":{},\"duration\":{:?}}}",json_string(object),vec3_json(*position),duration),WorldEvent3D::SetVisible{object,visible}=>format!("{{\"kind\":\"visible\",\"object\":{},\"visible\":{}}}",json_string(object),visible),WorldEvent3D::SetEpistemicState{object,state}=>format!("{{\"kind\":\"epistemic_state\",\"object\":{},\"state\":{}}}",json_string(object),json_string(state.as_str()))}}
fn json_string(value:&str)->String{let mut output=String::from("\"");for character in value.chars(){match character{'"'=>output.push_str("\\\""),'\\'=>output.push_str("\\\\"),'\n'=>output.push_str("\\n"),c if c.is_control()=>{let _=write!(output,"\\u{:04x}",c as u32);},c=>output.push(c)}}output.push('"');output}
#[cfg(test)]mod tests{use super::*;#[test]fn world_serializes_scene_hierarchy(){let w=World3D::the_door();let j=w.to_json();assert!(j.contains("\"schema\":3"));assert!(j.contains("\"parent\":\"door_hinge\""));assert!(j.contains("wall_left"));}#[test]fn opening_rotates_hinge_and_moves_player(){let mut w=World3D::the_door();w.apply_action("open",false);assert!(w.events.iter().any(|e|matches!(e,WorldEvent3D::RotateY{object,..} if object=="door_hinge")));assert!(w.events.iter().any(|e|matches!(e,WorldEvent3D::MoveTo{object,..} if object=="player")));}#[test]fn evidence_and_reopening_mutate_world_state(){let mut w=World3D::the_door();w.apply_action("camera_has_blind_spot",false);assert_eq!(w.objects.iter().find(|o|o.id=="security_camera").unwrap().material.state,EpistemicVisualState::Evidence);w.apply_action("open",true);assert_eq!(w.objects.iter().find(|o|o.id=="door").unwrap().material.state,EpistemicVisualState::Reopened);assert!(w.objects.iter().find(|o|o.id=="reopened_marker").unwrap().visible);}}
