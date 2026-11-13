import {readFileSync,writeFileSync} from "node:fs";
import {resolve} from "node:path";
const root=resolve(import.meta.dir,"..");
const path=root+"/crates/web/src/preview.rs";
const original=readFileSync(path,"utf8");
const from="Some(&()),";
if(original.split(from).length!==2)throw Error("Renderer lifetime site changed");
async function run(command:string[],timeout:number){
  const child=Bun.spawn(command,{cwd:root,stdout:"pipe",stderr:"pipe"});
  const timer=setTimeout(()=>child.kill(),timeout);
  const [out,err,code]=await Promise.all([new Response(child.stdout).text(),new Response(child.stderr).text(),child.exited]);clearTimeout(timer);
  return {code,text:out+err};
}
try{
  writeFileSync(path,original.replace(from,"Some(&(view,)),"));
  const build=await run(["bun","run","build"],120000);
  if(build.code!==0)throw Error(build.text);
  const result=await run(["bunx","--bun","playwright","test","-g","without recreating WebGL"],30000);
  if(result.code===0||!["toBe(calls)","The cube must contain real colored sticker pixels"].some(assertion=>result.text.includes(assertion)))throw Error("Renderer lifetime mutation was not caught by the expected assertion:\n"+result.text);
  console.log("caught: renderer recreated on camera changes");
}finally{
  writeFileSync(path,original);
  const restored=await run(["bun","run","build"],120000);
  if(restored.code!==0)throw Error("Restoring the correct build failed:\n"+restored.text);
}

