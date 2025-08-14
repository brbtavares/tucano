# tucano-profitdll

Camada isolada de integração com a ProfitDLL (Nelógica) para o ecossistema **Tucano**.

Inclui:
- Tipos de eventos (`CallbackEvent`, `BookAction`, etc.)
- Identificadores (`AssetIdentifier`, `AccountIdentifier`)
- Envio básico de ordens (`SendOrder`)
- Mock cross-plataform (Linux/Mac) para desenvolvimento
- (Opcional via feature `real_dll`) bindings FFI reais para Windows

## Uso
```toml
[dependencies]
tucano-profitdll = "0.1"
```

## Feature Flags
- `real_dll`: ativa FFI para DLL real (somente Windows, requer ProfitChart instalado)

## Segurança
Bindings FFI implicam riscos – valide inputs e trate códigos de retorno.

## Estabilidade da API
Enums e alguns eventos são marcados como `#[non_exhaustive]` para permitir adição de variantes sem quebrar dependentes. Considere usar correspondência com `_` ao fazer `match`.

## Roadmap Resumido
- Substituir structs simplificados de ordens por layout completo (`TConnectorSendOrder`, `TConnectorOrderOut`).
- Expandir callbacks de histórico / enumeração.
- Tests Windows com DLL real (feature `real_dll`) cobrindo ciclo completo de ordem.

Estado atual: base funcional + snapshot parcial de ordens (quando símbolo `GetOrderDetails` presente) pronto para integração principal.

## Licença
MIT
