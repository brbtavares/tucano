# brokers

Camada de abstração para corretoras (brokers) integrada ao ecossistema Toucan.

## Objetivos
- Identificação padronizada de corretoras (código operacional + nome oficial)
- Metadados de certificação (ex: Selo PQO B3)
- Associação de modelos de custo (ex: corretagem, custódia, emolumentos) por instrumento
- Extensível para múltiplos mercados / geografias

## Conceitos
- `BrokerId`: identificador curto (slug) estável utilizado internamente
- `BrokerCode`: código operacional usado na B3 (quando aplicável)
- `BrokerName`: nome oficial publicado
- `BrokerCertification`: enum de certificações reconhecidas
- `CostModel`: coleção de funções/estratégias para calcular custos de execução

## Próximos passos
1. Integrar `brokers` com crates `execution` e `markets` (associar ExchangeId -> BrokerId quando aplicável)
2. Enriquecer modelos de custo com camadas (fixo + variável + por contrato)
3. Persistência e atualização dinâmica de custos (ex: carregar JSON/YAML)
