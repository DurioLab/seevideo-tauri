将你的 RSA 密钥放这里：

- `license_pub.pem`（给激活码生成脚本使用）
- `license_priv.pem`（给桌面应用解密使用）

你可以先用 OpenSSL 生成：

openssl genrsa -out license_priv.pem 2048
openssl rsa -in license_priv.pem -pubout -out license_pub.pem
