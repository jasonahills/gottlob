import init, { run_app } from "../pkg/gottlob_web.js"
async function main() {
  await init("/pkg/gottlob_web.wasm");
  run_app();
}
main()