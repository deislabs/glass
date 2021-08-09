async function get() {
  return 42;
}

// (async () => {
//   console.log(await get());
// })();

async function main() {
  var x = await get();

  let uint8Array = new Uint8Array([72, 101, 108, 108, 111]);
  console.log(new TextDecoder().decode(uint8Array));

  const encoder = new TextEncoder();
  const view = encoder.encode("€");
  console.log(view); // Uint8Array(3) [226, 130, 172]

  console.log("This is the main function + value " + x);
}

function test(i) {
  console.log("Value of parameter: " + i);
}
