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

## Licença
MIT
