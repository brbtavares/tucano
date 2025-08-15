# profitdll

Renomeação da antiga crate `tucano-profitdll` para seguir convenção sem prefixo no diretório.

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
profitdll = { path = "../profitdll" }
```

(Quando publicado no crates.io substituir por versão.)

## Feature Flags
- `real_dll`: ativa FFI para DLL real (somente Windows, requer ProfitChart instalado)

## Segurança
Bindings FFI implicam riscos – valide inputs e trate códigos de retorno.

## Estabilidade da API
Enums e alguns eventos são marcados como `#[non_exhaustive]`.

Veja `MANUAL.md` para documentação extensa extraída e normalizada.
