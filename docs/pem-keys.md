# 1. Gerar chave privada
openssl genpkey -algorithm ED25519 -out private.pem

# 2. Extrair chave pública
openssl pkey -in private.pem -pubout -out public.pem
