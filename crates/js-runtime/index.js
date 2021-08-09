async function getAnswer() {
  return 42;
}

async function main() {
  console.log("content-type: text/plain\n");

  console.log("Yup, writing WAGI handlers using JavaScript!\n");

  console.log("Oh look, async functions work too!\n");
  console.log(
    "The answer to life, the universe, and everything: " + (await getAnswer())
  );

  console.log("How about text encoders and decoders?\n");
  let uint8Array = new Uint8Array([72, 101, 108, 108, 111]);
  console.log(new TextDecoder().decode(uint8Array));

  const encoder = new TextEncoder();
  const view = encoder.encode("€");
  console.log(view); // Uint8Array(3) [226, 130, 172]
}

function test(i) {
  console.log("Value of parameter: " + i);
}
