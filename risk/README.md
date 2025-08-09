# Toucan Risk

> Camada de validação e controle de risco antes de ordens chegarem ao venue (B3 inicialmente).

## 🎯 Papel
A crate **risk** implementa verificações para proteger capital, assegurar conformidade de limites e evitar execução incorreta. Atua como filtro entre geração de sinal (strategy) e submissão (execution).

| Responsabilidade | Descrição |
|------------------|-----------|
| Checks Atômicos | `check/` contém validadores (ex: tamanho máximo de posição) |
| Composição | Estrutura para combinar múltiplos checks sequencialmente |
| Resultado Fortemente Tipado | Tipos `RiskApproved<T>` / `RiskRefused<T>` encapsulam decisão |
| Erros Semânticos | Mapeamento claro de motivos de bloqueio |

## 🔑 Principais Conceitos
- `RiskManager` (futuro trait / struct agregadora) – Orquestra avaliação.
- `check::*` – Módulos individuais para cada política (placeholder inicial).
- `RiskApproved<T>` / `RiskRefused<T>` – Wrappers garantindo que somente fluxos aprovados avancem.

## 🔗 Interdependências
| Depende de | Motivo |
|------------|-------|
| `markets` | Identificação de instrumentos / ativos |
| `execution` | Acesso a ordens / posições vigentes |
| `analytics` (futuro) | Volatilidade / métricas dinâmicas para limites adaptativos |

| Consumido por | Uso |
|---------------|-----|
| `core` | Gate de pré‑execução |
| `strategy` | Ajuste de posição baseado em resposta de risco |

## ✅ Concluído
- Estrutura inicial de tipos aprovados / recusados.
- Esqueleto de checks básicos.

## 🧪 Parcial
- Lista real de checks (exposição notional, stop global, perda diária) não implementada.
- Integração com métricas runtime.

## 🚧 Pendências
- Política de agregação (primeiro falha vs coletar todos os motivos).
- Modo simulação vs produção (thresholds distintos).
- Auditoria de decisões (log estruturado / métricas de bloqueio).

## 🇧🇷 Contexto B3
Checks irão incluir: limites por contrato futuro, margem mínima, limites de oscilação (circuit breaker local), filtros de horário (leilões, after-market) e regras específicas da bolsa.

## Exemplo (conceitual)
```rust
// pseudo-código
if risk_manager.validate(&order).is_ok() {
   submit(order)
}
```
