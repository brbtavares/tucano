
# brokers

Abstraction layer for brokers integrated into the Tucano ecosystem.

## Objectives
- Standardized identification of brokers (operational code + official name)
- Certification metadata (e.g., B3 PQO Seal)
- Association of cost models (e.g., brokerage, custody, fees) per instrument
- Extensible for multiple markets/geographies

## Concepts
- `BrokerId`: short, stable identifier (slug) used internally
- `BrokerCode`: operational code used on B3 (when applicable)
- `BrokerName`: official published name
- `BrokerCertification`: enum of recognized certifications
- `CostModel`: collection of functions/strategies to calculate execution costs

## Next Steps
1. Integrate `brokers` with the `execution` and `markets` crates (associate ExchangeId -> BrokerId when applicable)
2. Enrich cost models with layers (fixed + variable + per contract)
3. Persistence and dynamic update of costs (e.g., load JSON/YAML)
