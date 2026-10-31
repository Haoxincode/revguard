"use strict";
function greet(name){return `hello, ${name}`;}
function add(a,b){return a+b;}
function main(){const msg=greet("revguard");const sum=add(2,40);return{msg,sum,ok:msg==="hello, revguard"&&sum===42};}
const __rg_out=main();
if(typeof console!=="undefined")console.log(JSON.stringify(__rg_out));
if(typeof process!=="undefined"&&!__rg_out.ok)process.exit(1);
