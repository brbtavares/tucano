# Tucano Examples (Curadoria em andamento)

Organização:
- mock/: exemplos reprodutíveis (sem credenciais)
- live/: dependem de PROFIT_USER/PROFIT_PASSWORD

Exemplos atuais:
- mock_minimal
- live_minimal (flag opcional `--allow-order` para enviar 1 ordem de mercado)

Executar:
```
cargo run -p tucano-examples --bin mock_minimal
cargo run -p tucano-examples --bin live_minimal
cargo run -p tucano-examples --bin live_minimal -- --allow-order  # habilita envio
```

Variáveis (live): PROFIT_USER, PROFIT_PASSWORD, opcional PROFIT_ACTIVATION_KEY, LIVE_TICKER, LIVE_EXCHANGE, LIVE_ACCOUNT_ID, LIVE_BROKER.

Roadmap:
- [x] mock mínimo
- [x] live mínimo
- [ ] envio de ordem protegido
- [ ] estratégia mock
- [ ] estratégia híbrida
