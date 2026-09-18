import init, { WebSession, caveat_map } from './pkg/caveat_runtime.js';

const DEG = Math.PI / 180;
const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
const ease = t => 1 - Math.pow(1 - t, 3);
const lerp = (a, b, t) => a + (b - a) * t;
const mix3 = (a, b, t) => a.map((v, i) => lerp(v, b[i], t));
const safe = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));

function mat4Identity(){return new Float32Array([1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1])}
function mat4Mul(a,b){const o=new Float32Array(16);for(let r=0;r<4;r++)for(let c=0;c<4;c++)for(let k=0;k<4;k++)o[c*4+r]+=a[k*4+r]*b[c*4+k];return o}
function mat4Translate(x,y,z){const m=mat4Identity();m[12]=x;m[13]=y;m[14]=z;return m}
function mat4Scale(x,y,z){const m=mat4Identity();m[0]=x;m[5]=y;m[10]=z;return m}
function mat4RotX(a){const c=Math.cos(a),s=Math.sin(a);return new Float32Array([1,0,0,0,0,c,s,0,0,-s,c,0,0,0,0,1])}
function mat4RotY(a){const c=Math.cos(a),s=Math.sin(a);return new Float32Array([c,0,-s,0,0,1,0,0,s,0,c,0,0,0,0,1])}
function mat4RotZ(a){const c=Math.cos(a),s=Math.sin(a);return new Float32Array([c,s,0,0,-s,c,0,0,0,0,1,0,0,0,0,1])}
function mat4Perspective(fov,aspect,near,far){const f=1/Math.tan(fov*DEG/2),o=new Float32Array(16);o[0]=f/aspect;o[5]=f;o[10]=(far+near)/(near-far);o[11]=-1;o[14]=(2*far*near)/(near-far);return o}
function vsub(a,b){return[a[0]-b[0],a[1]-b[1],a[2]-b[2]]}
function vcross(a,b){return[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]}
function vnorm(a){const n=Math.hypot(a[0],a[1],a[2])||1;return a.map(v=>v/n)}
function mat4LookAt(eye,target,up=[0,1,0]){const z=vnorm(vsub(eye,target)),x=vnorm(vcross(up,z)),y=vcross(z,x);return new Float32Array([x[0],y[0],z[0],0,x[1],y[1],z[1],0,x[2],y[2],z[2],0,-(x[0]*eye[0]+x[1]*eye[1]+x[2]*eye[2]),-(y[0]*eye[0]+y[1]*eye[1]+y[2]*eye[2]),-(z[0]*eye[0]+z[1]*eye[1]+z[2]*eye[2]),1])}
function transformMatrix(o){const p=o.position||[0,0,0],r=(o.rotation||[0,0,0]).map(v=>v*DEG),s=o.scale||[1,1,1];return mat4Mul(mat4Translate(...p),mat4Mul(mat4RotY(r[1]),mat4Mul(mat4RotX(r[0]),mat4Mul(mat4RotZ(r[2]),mat4Scale(...s)))))}

function cubeMesh(){
  const p=[-1,-1,1,1,-1,1,1,1,1,-1,1,1,1,-1,-1,-1,-1,-1,-1,1,-1,1,1,-1,-1,1,1,1,1,1,1,1,-1,-1,1,-1,-1,-1,-1,-1,-1,1,-1,1,1,1,1,-1,-1,-1,-1,1,-1,-1,1,1,-1,1,1,-1,1,-1,-1,1,-1,1,-1,-1,-1,-1,1,-1];
  const n=[0,0,1,0,0,1,0,0,1,0,0,1,0,0,-1,0,0,-1,0,0,-1,0,0,-1,0,1,0,0,1,0,0,1,0,0,1,0,0,-1,0,0,-1,0,0,-1,0,0,-1,0,1,0,0,1,0,0,1,0,0,1,0,0,-1,0,0,-1,0,0,-1,0,0,-1,0,0];
  const i=[];for(let f=0;f<6;f++){const b=f*4;i.push(b,b+1,b+2,b,b+2,b+3)}return{p:new Float32Array(p),n:new Float32Array(n),i:new Uint16Array(i)}
}
function sphereMesh(lat=10,lon=14){const p=[],n=[],i=[];for(let y=0;y<=lat;y++){const v=y/lat,phi=v*Math.PI;for(let x=0;x<=lon;x++){const u=x/lon,th=u*Math.PI*2,s=Math.sin(phi),px=Math.cos(th)*s,py=Math.cos(phi),pz=Math.sin(th)*s;p.push(px,py,pz);n.push(px,py,pz)}}for(let y=0;y<lat;y++)for(let x=0;x<lon;x++){const a=y*(lon+1)+x,b=a+lon+1;i.push(a,b,a+1,b,b+1,a+1)}return{p:new Float32Array(p),n:new Float32Array(n),i:new Uint16Array(i)}}
function cylinderMesh(seg=16){const p=[],n=[],i=[];for(let y=0;y<=1;y++){for(let s=0;s<=seg;s++){const a=s/seg*Math.PI*2,x=Math.cos(a),z=Math.sin(a);p.push(x,y*2-1,z);n.push(x,0,z)}}for(let y=0;y<1;y++)for(let s=0;s<seg;s++){const a=y*(seg+1)+s,b=a+seg+1;i.push(a,b,a+1,b,b+1,a+1)}const addCap=(yy,ny)=>{const center=p.length/3;p.push(0,yy,0);n.push(0,ny,0);const start=p.length/3;for(let s=0;s<=seg;s++){const a=s/seg*Math.PI*2;p.push(Math.cos(a),yy,Math.sin(a));n.push(0,ny,0)}for(let s=0;s<seg;s++){if(ny>0)i.push(center,start+s,start+s+1);else i.push(center,start+s+1,start+s)}};addCap(1,1);addCap(-1,-1);return{p:new Float32Array(p),n:new Float32Array(n),i:new Uint16Array(i)}}

class Renderer3D{
  constructor(canvas,manifest){
    this.canvas=canvas;this.gl=canvas.getContext('webgl',{antialias:true,alpha:false});if(!this.gl)throw new Error('WebGL unavailable');
    this.manifest=manifest;this.objects=new Map();this.meshes={cube:cubeMesh(),sphere:sphereMesh(),cylinder:cylinderMesh()};
    this.camera={position:[...(manifest.camera?.position||[0,4,14])],target:[...(manifest.camera?.target||[0,1,0])],fov:manifest.camera?.fov||58};
    this.cameraTween=null;this.userYaw=0;this.userPitch=0;this.drag=null;this.last=performance.now();
    this.initGL();this.loadObjects(manifest.objects||[]);this.bindPointers();this.draw();requestAnimationFrame(t=>this.frame(t));
  }
  initGL(){
    const gl=this.gl;
    const vs=`attribute vec3 aPos;attribute vec3 aNormal;uniform mat4 uVP;uniform mat4 uModel;varying vec3 vNormal;varying vec3 vWorld;void main(){vec4 w=uModel*vec4(aPos,1.0);vWorld=w.xyz;vNormal=(uModel*vec4(aNormal,0.0)).xyz;gl_Position=uVP*w;}`;
    const fs=`precision mediump float;uniform vec3 uColor;uniform vec3 uFog;uniform float uEmissive;uniform float uOpacity;uniform vec3 uCamera;varying vec3 vNormal;varying vec3 vWorld;void main(){vec3 n=normalize(vNormal);vec3 sun=normalize(vec3(-0.35,0.8,0.42));float d=max(dot(n,sun),0.0);float hemi=0.48+0.30*max(n.y,0.0);vec3 c=uColor*(hemi+d*0.42)+uColor*uEmissive;float dist=distance(vWorld,uCamera);float fog=clamp((dist-15.0)/34.0,0.0,0.62);c=mix(c,uFog,fog);gl_FragColor=vec4(c,uOpacity);}`;
    const sh=(type,src)=>{const s=gl.createShader(type);gl.shaderSource(s,src);gl.compileShader(s);if(!gl.getShaderParameter(s,gl.COMPILE_STATUS))throw new Error(gl.getShaderInfoLog(s));return s};
    this.program=gl.createProgram();gl.attachShader(this.program,sh(gl.VERTEX_SHADER,vs));gl.attachShader(this.program,sh(gl.FRAGMENT_SHADER,fs));gl.linkProgram(this.program);if(!gl.getProgramParameter(this.program,gl.LINK_STATUS))throw new Error(gl.getProgramInfoLog(this.program));gl.useProgram(this.program);
    this.loc={pos:gl.getAttribLocation(this.program,'aPos'),normal:gl.getAttribLocation(this.program,'aNormal'),vp:gl.getUniformLocation(this.program,'uVP'),model:gl.getUniformLocation(this.program,'uModel'),color:gl.getUniformLocation(this.program,'uColor'),fog:gl.getUniformLocation(this.program,'uFog'),em:gl.getUniformLocation(this.program,'uEmissive'),op:gl.getUniformLocation(this.program,'uOpacity'),camera:gl.getUniformLocation(this.program,'uCamera')};
    for(const m of Object.values(this.meshes)){m.pb=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,m.pb);gl.bufferData(gl.ARRAY_BUFFER,m.p,gl.STATIC_DRAW);m.nb=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,m.nb);gl.bufferData(gl.ARRAY_BUFFER,m.n,gl.STATIC_DRAW);m.ib=gl.createBuffer();gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER,m.ib);gl.bufferData(gl.ELEMENT_ARRAY_BUFFER,m.i,gl.STATIC_DRAW)}
    gl.enable(gl.DEPTH_TEST);
  }
  loadObjects(list){for(const raw of list)this.objects.set(raw.id,{visible:raw.visible!==false,emissive:raw.emissive||0,opacity:raw.opacity??1,...raw})}
  setVisible(id,v){const o=this.objects.get(id);if(o)o.visible=v}
  setEmphasis(ids=[]){const set=new Set(ids);for(const o of this.objects.values())o._boost=set.has(o.id)?0.42:0}
  animateCamera(to,duration=1.1){if(!to)return;const pos=to.position||this.camera.position,target=to.target||this.camera.target;this.cameraTween={fromP:[...this.camera.position],toP:[...pos],fromT:[...this.camera.target],toT:[...target],t:0,d:duration}}
  bindPointers(){this.canvas.addEventListener('pointerdown',e=>{this.drag={x:e.clientX,y:e.clientY};this.canvas.setPointerCapture?.(e.pointerId)});this.canvas.addEventListener('pointermove',e=>{if(!this.drag)return;this.userYaw+=(e.clientX-this.drag.x)*.004;this.userPitch=clamp(this.userPitch+(e.clientY-this.drag.y)*.003,-.35,.35);this.drag={x:e.clientX,y:e.clientY}});this.canvas.addEventListener('pointerup',()=>this.drag=null);this.canvas.addEventListener('pointercancel',()=>this.drag=null)}
  update(dt){if(this.cameraTween){this.cameraTween.t=Math.min(this.cameraTween.d,this.cameraTween.t+dt);const q=ease(this.cameraTween.t/this.cameraTween.d);this.camera.position=mix3(this.cameraTween.fromP,this.cameraTween.toP,q);this.camera.target=mix3(this.cameraTween.fromT,this.cameraTween.toT,q);if(this.cameraTween.t>=this.cameraTween.d)this.cameraTween=null}}
  frame(now){const dt=Math.min(.05,(now-this.last)/1000);this.last=now;this.update(dt);this.draw();requestAnimationFrame(t=>this.frame(t))}
  draw(){const gl=this.gl,d=Math.min(devicePixelRatio||1,2),w=Math.max(1,Math.floor(this.canvas.clientWidth*d)),h=Math.max(1,Math.floor(this.canvas.clientHeight*d));if(this.canvas.width!==w||this.canvas.height!==h){this.canvas.width=w;this.canvas.height=h}gl.viewport(0,0,w,h);const clear=this.manifest.atmosphere?.clear||[.035,.055,.075];gl.clearColor(clear[0],clear[1],clear[2],1);gl.clear(gl.COLOR_BUFFER_BIT|gl.DEPTH_BUFFER_BIT);gl.useProgram(this.program);
    let eye=[...this.camera.position],target=[...this.camera.target];if(this.userYaw||this.userPitch){const v=vsub(eye,target),r=Math.hypot(v[0],v[2]),ang=Math.atan2(v[0],v[2])+this.userYaw;eye=[target[0]+Math.sin(ang)*r,target[1]+v[1]+this.userPitch*r,target[2]+Math.cos(ang)*r]}
    const vp=mat4Mul(mat4Perspective(this.camera.fov,w/h,.1,100),mat4LookAt(eye,target));gl.uniformMatrix4fv(this.loc.vp,false,vp);gl.uniform3fv(this.loc.fog,this.manifest.atmosphere?.fog||clear);gl.uniform3fv(this.loc.camera,eye);
    const opaque=[],trans=[];for(const o of this.objects.values())if(o.visible)(o.opacity??1)<.999?trans.push(o):opaque.push(o);for(const o of opaque)this.drawObject(o);if(trans.length){gl.enable(gl.BLEND);gl.blendFunc(gl.SRC_ALPHA,gl.ONE_MINUS_SRC_ALPHA);gl.depthMask(false);for(const o of trans)this.drawObject(o);gl.depthMask(true);gl.disable(gl.BLEND)}
  }
  drawObject(o){const gl=this.gl,m=this.meshes[o.primitive||'cube']||this.meshes.cube;gl.bindBuffer(gl.ARRAY_BUFFER,m.pb);gl.enableVertexAttribArray(this.loc.pos);gl.vertexAttribPointer(this.loc.pos,3,gl.FLOAT,false,0,0);gl.bindBuffer(gl.ARRAY_BUFFER,m.nb);gl.enableVertexAttribArray(this.loc.normal);gl.vertexAttribPointer(this.loc.normal,3,gl.FLOAT,false,0,0);gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER,m.ib);gl.uniformMatrix4fv(this.loc.model,false,transformMatrix(o));gl.uniform3fv(this.loc.color,o.color||[.7,.7,.7]);gl.uniform1f(this.loc.em,(o.emissive||0)+(o._boost||0));gl.uniform1f(this.loc.op,o.opacity??1);gl.drawElements(gl.TRIANGLES,m.i.length,gl.UNSIGNED_SHORT,0)}
  get idle(){return !this.cameraTween}
}

export async function bootCaveat3D(manifest, root=document){
  await init();
  const canvas=root.querySelector('#c3d');
  const gameEl=root.querySelector('#game3d');
  const statusEl=root.querySelector('#status3d');
  const placeEl=root.querySelector('#place3d');
  if(!canvas||!gameEl||!statusEl)throw new Error('CAVEAT 3D shell is incomplete');
  const response=await fetch(manifest.source);if(!response.ok)throw new Error(`scenario HTTP ${response.status}`);const source=await response.text();
  const rawMap=JSON.parse(caveat_map(source));if(rawMap.kind==='error')throw new Error(rawMap.message);const map=rawMap;
  const displays=new Map();for(const s of map.symbols||[])if(s.display)displays.set(s.name,s.display);for(const a of map.actions||[])if(a.display)displays.set(a.id,a.display);
  let session=new WebSession(source),history=[],lastChoice=null;
  const renderer=new Renderer3D(canvas,manifest);window.__caveat3dIdle=()=>renderer.idle;
  const pretty=id=>displays.get(id)||String(id).replaceAll('_',' ').replace(/\b\w/g,x=>x.toUpperCase());
  const caveatText=id=>manifest.caveats?.[id]||'This evidence is useful, but it is not absolute.';
  function log(kind,text){history.push({kind,text});root.querySelector('#trail3d').innerHTML=history.map((x,i)=>`<div class="trail-row"><span>${i+1}</span><div><b>${safe(x.kind)}</b> ${safe(x.text)}</div></div>`).join('')}
  function applyPresentation(id){const p=manifest.actions?.[id];if(!p)return;renderer.setEmphasis(p.emphasize||[]);if(p.camera)renderer.animateCamera(p.camera,p.duration||1.1);for(const x of p.reveal||[])renderer.setVisible(x,true);for(const x of p.hide||[])renderer.setVisible(x,false);if(placeEl&&p.place)placeEl.textContent=p.place}
  function copyFor(name){return manifest.interactions?.[name]||{eyebrow:'CAVEAT',title:'Choose what to do next',body:'The world is waiting.'}}
  function render(){let p;try{p=JSON.parse(session.pending())}catch(e){return fail(e)}if(p.kind==='error')return fail(p.message);if(p.kind==='complete')return ending();const copy=copyFor(p.name),investigate=p.kind==='investigate';statusEl.textContent=`CAVEAT · attention ${p.budget}`;const buttons=(p.options||[]).map(id=>`<button class="choice3d" type="button" data-id="${safe(id)}"><b>${safe(pretty(id))}</b><small>${safe(manifest.hints?.[id]||'')}</small></button>`).join('');gameEl.innerHTML=`<div class="eyebrow">${safe(copy.eyebrow||(investigate?'Investigation':'Decision'))}</div><h1>${safe(copy.title)}</h1><p>${safe(copy.body||'')}</p><div class="meta"><span>Attention ${p.budget}</span><span>${investigate?`Investigate · cost ${p.cost}`:'Commit under uncertainty'}</span></div><div class="choices3d">${buttons}</div>`;gameEl.querySelectorAll('.choice3d').forEach(b=>b.addEventListener('click',()=>apply(b.dataset.id,p.kind)))}
  function apply(id,kind){gameEl.querySelectorAll('button').forEach(b=>b.disabled=true);try{session.apply(id);applyPresentation(id);log(kind==='investigate'?'Investigated':'Committed',pretty(id));if(kind==='choice')lastChoice=id;const discoveries=JSON.parse(session.discoveries()),commitment=JSON.parse(session.commitment());if(discoveries.length)return discovery(id,discoveries);if(commitment)return commitmentView(id,commitment);render()}catch(e){fail(e)}}
  function discovery(id,discoveries){const first=pretty(discoveries[0].evidence);const rows=discoveries.map(d=>`<div class="finding"><b>${d.relation==='opposes'?'Weakens':'Supports'}</b> ${safe(pretty(d.target))}</div>`).join('');gameEl.innerHTML=`<div class="eyebrow">Clue found</div><h1>${safe(first)}</h1><p>${safe(manifest.discoveryText?.[id]||first)}</p><div class="findings">${rows}</div><div class="caveat"><b>Caveat examined</b>${safe(caveatText(id))}</div><button class="primary" type="button" id="continue3d">Keep going</button>`;gameEl.querySelector('#continue3d').addEventListener('click',render);statusEl.textContent='CAVEAT · evidence updated'}
  function commitmentView(id,c){const reopened=(c.reopened_by||[]).length>0,cause=reopened?c.reopened_by[0]:null;gameEl.innerHTML=`<div class="eyebrow">${reopened?'Commitment reopened':'Commitment recorded'}</div><h1>${reopened?'That changes things.':'Decision made.'}</h1><p>${safe(manifest.commitmentText?.[id]||(reopened?'New evidence makes the earlier move worth revisiting.':'The decision is recorded without pretending uncertainty disappeared.'))}</p><div class="caveat"><b>${reopened?'Why it reopened':'Uncertainty retained'}</b>${safe(cause?caveatText(cause):`${(c.retained||[]).length} caveat(s) remain attached to this commitment.`)}</div><button class="primary" type="button" id="continue3d">${reopened?'Revise the plan':'See what happens'}</button>`;gameEl.querySelector('#continue3d').addEventListener('click',render);log(reopened?'Reopened':'Retained',reopened?pretty(cause):`${(c.retained||[]).length} caveat(s)`);statusEl.textContent=reopened?'CAVEAT · reopened':'CAVEAT · commitment held'}
  function ending(){const end=manifest.endings?.[lastChoice]||manifest.ending||{title:'Complete',body:'The route is complete.'};if(end.presentation)applyPresentation(end.presentation);for(const x of end.reveal||[])renderer.setVisible(x,true);if(end.camera)renderer.animateCamera(end.camera,end.duration||1.3);if(placeEl&&end.place)placeEl.textContent=end.place;statusEl.textContent='CAVEAT · complete';gameEl.innerHTML=`<div class="eyebrow">Story complete</div><h1>${safe(end.title)}</h1><p>${safe(end.body)}</p><div class="ending-chip">${safe(end.badge||'uncertainty preserved, then revised')}</div><button class="primary" type="button" id="again3d">Play again</button>`;gameEl.querySelector('#again3d').addEventListener('click',reset);log('Resolved',end.log||'The story reached an ending while preserving the decision trail.')}
  function reset(){session=new WebSession(source);history=[];lastChoice=null;for(const o of manifest.objects||[])renderer.setVisible(o.id,o.visible!==false);renderer.camera.position=[...(manifest.camera?.position||[0,4,14])];renderer.camera.target=[...(manifest.camera?.target||[0,1,0])];renderer.userYaw=0;renderer.userPitch=0;renderer.setEmphasis([]);if(placeEl)placeEl.textContent=manifest.place||'';root.querySelector('#trail3d').innerHTML='<div class="trail-row"><span>0</span><div><b>Story loaded.</b> No decisions yet.</div></div>';render()}
  function fail(e){statusEl.textContent='CAVEAT · error';gameEl.innerHTML=`<div class="error"><b>The 3D story could not continue.</b><br>${safe(e?.message||e)}</div>`}
  if(placeEl)placeEl.textContent=manifest.place||'';render();return{renderer,map,session};
}
