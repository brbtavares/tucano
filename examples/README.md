# Tucano Examples (Curadoria em andamento)

Organização:
- mock/: exemplos reprodutíveis (sem credenciais)
- live/: dependem de PROFIT_USER/PROFIT_PASSWORD

Exemplos atuais:
- mock_minimal
- live_login_single_ticker

Executar:
```
cargo run -p tucano-examples --bin mock_minimal
cargo run -p tucano-examples --bin live_login_single_ticker
```

Variáveis (live): PROFIT_USER, PROFIT_PASSWORD, opcional PROFIT_ACTIVATION_KEY, LIVE_TICKER, LIVE_EXCHANGE, LIVE_ACCOUNT_ID, LIVE_BROKER.

Roadmap:
- [x] mock mínimo
- [x] live login + 1 ticker
- [ ] envio de ordem protegido
- [ ] estratégia mock
- [ ] estratégia híbrida
