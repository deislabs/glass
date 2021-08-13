async function getAnswer() {
  return 42;
}

async function handler(req, res) {
  console.log("Value of body: " + new TextDecoder().decode(req.body));
  console.log("Value of method: " + req.method);

  console.log(
    "The answer to life, the universe, and everything: " + (await getAnswer())
  );

  res.status = 418;
}
