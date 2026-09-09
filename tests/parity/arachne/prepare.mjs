// Test-data preparation only; pinned upstream profiles are read, never edited.
import fs from 'node:fs';
import path from 'node:path';
const root = '/home/indexyz/ares/OrcaSlicer/resources/profiles';
const out = process.argv[2];
fs.mkdirSync(out, { recursive: true });
function index(dir) {
  const result = new Map();
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const file = path.join(dir, entry.name);
    if (entry.isDirectory()) for (const [k, v] of index(file)) result.set(k, v);
    else if (file.endsWith('.json')) {
      const value = JSON.parse(fs.readFileSync(file));
      if (value.name) result.set(value.name, value);
    }
  }
  return result;
}
function flatten(index, name) {
  const value = index.get(name);
  if (!value) throw new Error(`Missing parent/preset ${name}`);
  const result = { ...(value.inherits ? flatten(index, value.inherits) : {}), ...value };
  delete result.inherits;
  return result;
}
const vendor = 'Snapmaker';
const machine = flatten(index(`${root}/${vendor}/machine`), 'Snapmaker A250 (0.4 nozzle)');
const processPreset = flatten(index(`${root}/${vendor}/process`), machine.default_print_profile);
const filaments = new Map([...index(`${root}/OrcaFilamentLibrary/filament`), ...index(`${root}/${vendor}/filament`)]);
const filament = flatten(filaments, machine.default_filament_profile[0]);
for (const [kind, value] of [['machine', machine], ['process', processPreset], ['filament', filament]]) {
  fs.writeFileSync(`${out}/${kind}.json`, JSON.stringify(value, null, 2) + '\n');
}
// Extruded rectangular-cell unions: no interior faces, upward top/downward bottom.
function prism(name, xs, ys, occupied) {
  const triangles = [];
  const quad = (a,b,c,d) => triangles.push([a,b,c], [a,c,d]);
  for (let i=0; i<xs.length-1; i++) for (let j=0; j<ys.length-1; j++) {
    if (!occupied(i,j)) continue;
    const a=[xs[i],ys[j],0], b=[xs[i+1],ys[j],0];
    const c=[xs[i+1],ys[j+1],0], d=[xs[i],ys[j+1],0];
    const [A,B,C,D]=[a,b,c,d].map(([x,y])=>[x,y,2]);
    quad(d,c,b,a); quad(A,B,C,D);
    if (j===0 || !occupied(i,j-1)) quad(a,b,B,A);
    if (i===xs.length-2 || !occupied(i+1,j)) quad(b,c,C,B);
    if (j===ys.length-2 || !occupied(i,j+1)) quad(c,d,D,C);
    if (i===0 || !occupied(i-1,j)) quad(d,a,A,D);
  }
  fs.writeFileSync(`${out}/${name}.stl`, `solid ${name}\n` + triangles.map(t=>
    `facet normal 0 0 0\nouter loop\n${t.map(p=>`vertex ${p.join(' ')}\n`).join('')}endloop\nendfacet\n`
  ).join('') + `endsolid ${name}\n`);
}
prism('wide', [-10,10], [-8,8], ()=>true);
prism('neck', [-10,-2,2,10], [-5,-0.3,0.3,5], (i,j)=>i!==1 || j===1);
prism('hole', [-10,-3,3,10], [-10,-3,3,10], (i,j)=>i!==1 || j!==1);
console.log(JSON.stringify({ machine:machine.name, process:processPreset.name, filament:filament.name, wall_generator:processPreset.wall_generator }));
