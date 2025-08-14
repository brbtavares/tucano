# Contribuindo para o Tucano

Obrigado pelo interesse em contribuir! Algumas diretrizes rápidas:

## Fluxo de Trabalho
1. Fork & branch a partir de `main`.
2. Use branches descritivas: `feat/`, `fix/`, `chore/`, `docs/`.
3. Rode localmente (ou use o pre-commit): `cargo fmt && cargo clippy -- -D warnings && cargo test`.
4. Abra PR com descrição clara (contexto, motivação, mudanças). 
5. Mantenha commits coesos; faça squash se necessário.

## Estilo de Código
- Formatação: `rustfmt` (config em `rustfmt.toml`).
- Lints: clippy sem warnings.
- Evitar unwrap/expect fora de testes ou inicialização.

## Commits
Sugerido (sem ser rígido):
```
feat(core): adicionar suporte a X
fix(execution): corrigir erro em Y
chore(ci): atualizar workflow
```

## Testes
- Adicione casos para novas funcionalidades ou correções.
- Preferir testes unitários + integração mínima.

## Segurança
- Não incluir credenciais reais (B3, chaves API, etc.).
- Reporte vulnerabilidades de forma privada primeiro.

## Publicação de Crates
- Coordenada via script `./scripts/release.sh`.
- Versões seguem SemVer;
- Façade `tucano` agrega releases estáveis.

## Discussões
Abra issues para propostas maiores antes de implementar.

## Automatizando Checks Locais

Para evitar falhas no CI, habilite o hook pre-commit:

```
ln -sf ../../scripts/pre_commit.sh .git/hooks/pre-commit
```

Executado a cada commit:
- rustfmt (check)
- clippy (sem warnings)
- cargo-deny (se instalado)
- testes rápidos (`cargo test --lib`)

Rodar manualmente:
```
./scripts/pre_commit.sh
```

Instalar cargo-deny (uma vez):
```
cargo install cargo-deny --locked
```

Para acelerar durante desenvolvimento: use somente libs `cargo test --workspace --lib`; antes do PR rode testes completos se necessário.

Bem-vindo(a)! 🇧🇷
