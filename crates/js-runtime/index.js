async function getAnswer() {
  return 42;
}

async function handler(body_buffer) {
  console.log("Value of body: " + new TextDecoder().decode(body_buffer));
  console.log(
    "The answer to life, the universe, and everything: " + (await getAnswer())
  );
}
