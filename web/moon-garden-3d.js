const objects=[];
const add=o=>objects.push(o);
const box=(id,position,scale,color,extra={})=>add({id,primitive:'cube',position,scale,color,...extra});
const sphere=(id,position,scale,color,extra={})=>add({id,primitive:'sphere',position,scale,color,...extra});
const cyl=(id,position,scale,color,rotation=[0,0,0],extra={})=>add({id,primitive:'cylinder',position,scale,color,rotation,...extra});

box('ground',[0,-.55,-2],[18,.45,18],[.075,.18,.14]);
sphere('moon',[8.2,8.8,-18],[1.55,1.55,1.55],[.95,.9,.72],{emissive:.72});
sphere('pond',[-3.2,-.28,-3.1],[5.4,.18,3.75],[.045,.25,.31],{emissive:.08,opacity:.92});
sphere('pond_glow',[-3.2,-.20,-3.1],[5.0,.11,3.45],[.16,.47,.48],{emissive:.14,opacity:.42});

for(let i=0;i<11;i++){
  const z=8.1-i*1.15,x=Math.sin(i*.8)*.34;
  sphere(`stone_${i}`,[x,-.18,z],[.72,.12,.45],[.35,.39,.34]);
}

for(let i=0;i<9;i++){
  const t=i/8,x=-.7+i*.58,y=.02+Math.sin(t*Math.PI)*.72,z=-2.65;
  box(`bridge_plank_${i}`,[x,y,z],[.32,.10,1.12],[.42,.23,.14]);
}
for(const side of [-1,1]){
  for(let i=0;i<5;i++){
    const t=i/4,x=-.7+i*1.16,y=.5+Math.sin(t*Math.PI)*.72,z=-2.65+side*.68;
    cyl(`bridge_post_${side}_${i}`,[x,y,z],[.055,.52,.055],[.30,.16,.10]);
  }
  box(`bridge_rail_${side}`,[1.62,.92,-2.65+side*.68],[2.72,.045,.045],[.34,.17,.11],{rotation:[0,0,-2]});
}

box('pavilion_floor',[5.0,-.02,-4.8],[2.45,.16,2.2],[.24,.15,.11]);
for(const x of [3.15,6.85])for(const z of [-6.35,-3.25])cyl(`pavilion_post_${x}_${z}`,[x,1.55,z],[.095,1.55,.095],[.34,.17,.12]);
box('pavilion_beam_front',[5.0,2.92,-3.25],[2.05,.12,.12],[.39,.19,.13]);
box('pavilion_beam_back',[5.0,2.92,-6.35],[2.05,.12,.12],[.39,.19,.13]);
box('pavilion_roof_a',[4.15,3.22,-4.8],[2.15,.12,2.55],[.36,.12,.11],{rotation:[0,0,-12]});
box('pavilion_roof_b',[5.85,3.22,-4.8],[2.15,.12,2.55],[.36,.12,.11],{rotation:[0,0,12]});
box('tea_bench',[5.15,.55,-3.65],[1.25,.22,.42],[.43,.28,.18]);
box('tea_cushion',[5.15,.82,-3.65],[.92,.09,.33],[.55,.32,.25]);

box('greenhouse_floor',[-6.2,-.05,-4.6],[2.2,.12,2.4],[.18,.24,.22]);
for(const x of [-8.0,-4.4])for(const z of [-6.55,-2.65])cyl(`green_post_${x}_${z}`,[x,1.55,z],[.06,1.55,.06],[.34,.53,.52]);
for(const z of [-6.55,-2.65])box(`green_beam_${z}`,[-6.2,3.05,z],[1.95,.07,.07],[.34,.53,.52]);
box('green_roof_a',[-6.95,3.48,-4.6],[1.45,.06,2.2],[.25,.52,.52],{rotation:[0,0,-24],opacity:.82});
box('green_roof_b',[-5.45,3.48,-4.6],[1.45,.06,2.2],[.25,.52,.52],{rotation:[0,0,24],opacity:.82});
box('green_glass_back',[-6.2,1.55,-6.55],[1.8,1.5,.035],[.18,.45,.48],{opacity:.55,emissive:.05});
box('green_glass_side',[-8.0,1.55,-4.6],[.035,1.5,1.85],[.18,.45,.48],{opacity:.48,emissive:.05});

const lanternData=[[-1.1,2.25,1.2],[1.3,2.45,.3],[3.0,2.3,-1.3],[6.8,2.15,-2.1],[-4.1,2.1,-.8]];
lanternData.forEach((p,i)=>{
  cyl(`lantern_pole_${i}`,[p[0],1.12,p[2]],[.04,1.12,.04],[.18,.12,.08]);
  sphere(`lantern_${i}`,p,[.34,.48,.34],i%2?[.73,.25,.12]:[.66,.16,.14],{emissive:.45});
  sphere(`lantern_core_${i}`,[p[0],p[1],p[2]],[.13,.20,.13],[1,.66,.24],{emissive:.9});
});

const shrubs=[[-8,0,-8],[-6.7,0,-8.2],[-5,0,-8.1],[-1,0,-7.8],[1.1,0,-7.6],[3.4,0,-8.0],[7.8,0,-7.7],[8.7,0,-5.8],[8.3,0,-2.4],[-8.7,0,-1.6],[-8.4,0,1.0],[-6.8,0,2.2],[6.6,0,2.1],[8.1,0,1.1]];
shrubs.forEach((p,i)=>sphere(`shrub_${i}`,[p[0],.28,p[2]],[1.2,.65,1.05],i%2?[.06,.24,.14]:[.05,.19,.12]));
for(let i=0;i<9;i++){
  const a=i/9*Math.PI*2,x=6.7+Math.cos(a)*1.2,z=-5.9+Math.sin(a)*.85;
  sphere(`fern_${i}`,[x,.18,z],[.42,.25,.55],[.05,.31,.18]);
}
for(let i=0;i<18;i++){
  const x=-7+(i%6)*2.6,z=-7+Math.floor(i/6)*4.8,y=1.0+(i%4)*.43;
  sphere(`firefly_${i}`,[x,y,z],[.045,.045,.045],[.75,.94,.36],{emissive:1.2});
}

sphere('miso_body',[6.75,.42,-5.65],[.55,.42,.68],[.035,.04,.045],{visible:false});
sphere('miso_head',[6.45,.88,-5.32],[.40,.38,.38],[.035,.04,.045],{visible:false});
box('miso_ear_l',[6.19,1.18,-5.30],[.16,.26,.14],[.035,.04,.045],{rotation:[0,0,-18],visible:false});
box('miso_ear_r',[6.68,1.17,-5.30],[.16,.26,.14],[.035,.04,.045],{rotation:[0,0,18],visible:false});
cyl('miso_tail',[7.26,.66,-5.78],[.07,.55,.07],[.035,.04,.045],[55,0,18],{visible:false});
sphere('miso_eye_l',[6.27,.94,-4.97],[.045,.055,.035],[.94,.81,.32],{emissive:.35,visible:false});
sphere('miso_eye_r',[6.52,.94,-4.97],[.045,.055,.035],[.94,.81,.32],{emissive:.35,visible:false});

export const moonGarden3D={
  schema:'caveat3d/0.1',
  source:'./moon_garden.cav',
  place:'Moon Garden · Lantern Court',
  atmosphere:{clear:[.035,.06,.085],fog:[.05,.10,.12]},
  camera:{position:[0,4.5,13.8],target:[0,1.0,-2.7],fov:56},
  objects,
  interactions:{
    first_clue:{eyebrow:'Investigation',title:'Find a useful clue',body:'You have time to check one thing carefully before choosing a direction.'},
    first_move:{eyebrow:'Decision',title:'What do you trust?',body:'The garden offers three plausible routes. None is clean enough to call certain.'},
    second_clue:{eyebrow:'Investigation',title:'The first move had a catch',body:'The caveat mattered. Check one more signal before you recommit.'},
    final_move:{eyebrow:'Decision',title:'Make the revised call',body:'Act on what you know while keeping the remaining uncertainty attached.'}
  },
  hints:{
    bell_echo_distorts_direction:'A clear sound, but stone and water can throw it around.',
    wet_stones_hide_tracks:'Physical evidence, though the path is wet and busy.',
    greenhouse_glass_mirrors_movement:'Movement behind warm glass — or reflected lantern light.',
    follow_bell:'Head toward the moon bridge and the loudest chime.',
    follow_tracks:'Follow the damp stones beside the pond.',
    check_greenhouse:'Check the warm glasshouse first.',
    bridge_water_masks_sound:'Stand still under the bridge and separate water from quieter sounds.',
    cushion_may_hold_old_fur:'Check the still-warm pavilion cushion.',
    moths_follow_heat_not_cats:'Use the lantern moths as indirect evidence.',
    cross_moon_bridge:'Search the willow side beyond the bridge.',
    search_tea_pavilion:'Follow the converging clues behind the pavilion.',
    open_glasshouse:'Search the warm planters and benches.'
  },
  caveats:{
    bell_echo_distorts_direction:'The bell can echo between stone, water and the pavilion roof, so direction is uncertain.',
    wet_stones_hide_tracks:'Wet stone preserves some marks and erases others; the trail may be incomplete.',
    greenhouse_glass_mirrors_movement:'Lanterns and people reflect in greenhouse glass, so movement may not be inside.',
    bridge_water_masks_sound:'Running water can hide a soft purr.',
    cushion_may_hold_old_fur:'A black hair may be fresh, or it may have been there earlier.',
    moths_follow_heat_not_cats:'Moths react to heat and light; their movement is only indirect evidence.'
  },
  discoveryText:{
    bell_echo_distorts_direction:'The second chime lands farther right than the first. The bridge is no longer the obvious answer.',
    wet_stones_hide_tracks:'The pressed moss continues away from the wet stones toward the pavilion.',
    greenhouse_glass_mirrors_movement:'The pawprint is real — and it is on the outside of the greenhouse.'
  },
  commitmentText:{
    follow_bell:'An owl lifts from the lantern line and brushes a hanging bell. The sound was real, but not necessarily Miso.',
    follow_tracks:'The fountain splashes across the stones and erases the cleanest part of the trail.',
    check_greenhouse:'A lantern swings. The moving shape repeats in the glass. Some of what you saw was reflection.'
  },
  actions:{
    bell_echo_distorts_direction:{camera:{position:[1.4,3.2,8.1],target:[1.0,2.1,.1]},emphasize:['lantern_1','lantern_core_1'],duration:.9,place:'Moon Garden · Lantern Court'},
    wet_stones_hide_tracks:{camera:{position:[-1.2,2.3,5.7],target:[-3.2,0,-2.2]},emphasize:['stone_6','pond'],duration:.9,place:'Moon Garden · Pond Path'},
    greenhouse_glass_mirrors_movement:{camera:{position:[-1.5,3.5,6.1],target:[-6.2,1.4,-4.6]},emphasize:['green_glass_back','green_glass_side'],duration:1.0,place:'Moon Garden · Glasshouse'},
    follow_bell:{camera:{position:[2.0,2.7,3.6],target:[1.7,.9,-2.7]},emphasize:['bridge_plank_4','lantern_2'],duration:1.25,place:'Moon Garden · Moon Bridge'},
    follow_tracks:{camera:{position:[-.8,2.5,4.2],target:[-3.3,.1,-3.0]},emphasize:['pond','stone_7'],duration:1.2,place:'Moon Garden · Pond Path'},
    check_greenhouse:{camera:{position:[-2.2,2.8,2.0],target:[-6.1,1.3,-4.7]},emphasize:['green_glass_back'],duration:1.2,place:'Moon Garden · Glasshouse'},
    bridge_water_masks_sound:{camera:{position:[2.1,1.8,.5],target:[1.6,.65,-2.8]},emphasize:['bridge_plank_4','pond'],duration:.95,place:'Moon Garden · Moon Bridge'},
    cushion_may_hold_old_fur:{camera:{position:[3.1,2.15,.2],target:[5.15,.72,-3.7]},emphasize:['tea_cushion'],duration:1.0,place:'Moon Garden · Tea Pavilion'},
    moths_follow_heat_not_cats:{camera:{position:[3.0,3.0,2.0],target:[5.2,2.0,-3.0]},emphasize:['lantern_3','firefly_16'],duration:1.0,place:'Moon Garden · Lantern Line'},
    cross_moon_bridge:{camera:{position:[2.4,2.2,-.2],target:[2.0,.5,-3.1]},emphasize:['bridge_plank_4'],duration:1.2,place:'Moon Garden · Moon Bridge'},
    search_tea_pavilion:{camera:{position:[3.8,2.3,-.1],target:[5.6,.8,-4.8]},emphasize:['tea_cushion','fern_2'],duration:1.25,place:'Moon Garden · Tea Pavilion'},
    open_glasshouse:{camera:{position:[-2.5,2.3,-.3],target:[-6.2,1.2,-4.7]},emphasize:['greenhouse_floor'],duration:1.2,place:'Moon Garden · Glasshouse'}
  },
  endings:{
    search_tea_pavilion:{title:'There you are.',body:'Behind the tea pavilion, the ferns rustle once. Miso answers with a tiny purr and steps into the lantern light.',badge:'Miso found · uncertainty revised',place:'Moon Garden · Fern Nook',reveal:['miso_body','miso_head','miso_ear_l','miso_ear_r','miso_tail','miso_eye_l','miso_eye_r'],camera:{position:[4.3,1.65,-1.7],target:[6.5,.7,-5.45]},duration:1.45},
    cross_moon_bridge:{title:'A detour, then a purr.',body:'The willow bank is empty. From the quieter side of the bridge you finally hear Miso behind the pavilion.',badge:'Miso found · route revised',place:'Moon Garden · Moon Bridge',reveal:['miso_body','miso_head','miso_ear_l','miso_ear_r','miso_tail','miso_eye_l','miso_eye_r'],camera:{position:[2.0,2.0,-2.0],target:[6.4,.7,-5.4]},duration:1.55},
    open_glasshouse:{title:'Not inside. Right outside.',body:'The greenhouse is warm and empty. When the door closes, Miso appears on the pavilion path behind you.',badge:'Miso found · false lead resolved',place:'Moon Garden · Glasshouse',reveal:['miso_body','miso_head','miso_ear_l','miso_ear_r','miso_tail','miso_eye_l','miso_eye_r'],camera:{position:[-2.2,2.0,-1.0],target:[6.3,.7,-5.2]},duration:1.65}
  }
};
