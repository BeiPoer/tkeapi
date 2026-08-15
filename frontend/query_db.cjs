const { Client } = require('pg');
const client = new Client({ connectionString: 'postgres://tokensapi:tokensapi@localhost:5432/tokensapi' });
async function run() {
  await client.connect();
  const res = await client.query("SELECT id, username, role FROM users");
  console.log(res.rows);
  await client.end();
}
run().catch(console.error);
