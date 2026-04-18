#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

function arg(name, fallback = '') {
  const idx = process.argv.indexOf(name);
  if (idx < 0) return fallback;
  return process.argv[idx + 1] || fallback;
}

function base64url(buf) {
  return Buffer.from(buf)
    .toString('base64')
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/g, '');
}

function readCsv(file) {
  const lines = fs.readFileSync(file, 'utf8').split(/\r?\n/).filter(Boolean);
  if (!lines.length) throw new Error('CSV empty');
  const header = lines[0].trim();
  if (header !== 'email,password,status,registertime') {
    throw new Error('CSV header must be: email,password,status,registertime');
  }
  const out = [];
  for (let i = 1; i < lines.length; i++) {
    const line = lines[i].trim();
    if (!line) continue;
    const parts = line.split(',');
    if (parts.length < 4) continue;
    const [email, password, status, registertime] = parts;
    if (!email || !password) continue;
    out.push({ email: email.trim(), password: password.trim(), status: (status || '').trim(), registertime: (registertime || '').trim() });
  }
  return out;
}

function main() {
  const csvPath = arg('--csv');
  const pubKeyPath = arg('--public-key');
  const outPath = arg('--out', 'activation_code.txt');
  const exp = arg('--exp', '2099-12-31T23:59:59Z');
  const licenseId = arg('--license-id', 'LIC-' + Date.now());

  if (!csvPath || !pubKeyPath) {
    console.error('Usage: node generate_activation_code.js --csv <file> --public-key <pem> [--out <file>] [--exp <iso>] [--license-id <id>]');
    process.exit(1);
  }

  const accounts = readCsv(csvPath);
  if (!accounts.length) {
    throw new Error('No valid accounts in CSV');
  }

  const payload = {
    v: 1,
    license_id: licenseId,
    exp,
    created_at: new Date().toISOString(),
    accounts,
  };

  const payloadBytes = Buffer.from(JSON.stringify(payload), 'utf8');

  const dek = crypto.randomBytes(32);
  const iv = crypto.randomBytes(12);
  const cipher = crypto.createCipheriv('aes-256-gcm', dek, iv);
  const ciphertext = Buffer.concat([cipher.update(payloadBytes), cipher.final()]);
  const tag = cipher.getAuthTag();

  const pubPem = fs.readFileSync(pubKeyPath, 'utf8');
  const encDek = crypto.publicEncrypt(
    {
      key: pubPem,
      padding: crypto.constants.RSA_PKCS1_OAEP_PADDING,
      oaepHash: 'sha256',
    },
    dek,
  );

  const envelope = {
    v: 1,
    alg: 'RSA_OAEP_SHA256+AES_256_GCM',
    iv: base64url(iv),
    tag: base64url(tag),
    ek: base64url(encDek),
    ct: base64url(ciphertext),
  };

  const code = base64url(Buffer.from(JSON.stringify(envelope), 'utf8'));
  fs.writeFileSync(outPath, code + '\n', 'utf8');
  console.log('Activation code generated:', path.resolve(outPath));
  console.log('Accounts:', accounts.length);
}

main();
