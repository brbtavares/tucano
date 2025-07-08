# Toucan-Execution

Stream private account data from financial venues, and execute (live or mock) orders. Also provides
a feature rich MockExchange and MockExecutionClient to assist with backtesting and paper-trading.

**It is:**
* **Easy**: ExecutionClient trait provides a unified and simple language for interacting with exchanges.
* **Normalised**: Allow your strategy to communicate with every real or MockExchange using the same interface.
* **Extensible**: Toucan-Execution is highly extensible, making it easy to contribute by adding new exchange integrations!

**See: [`Toucan`], [`Toucan-Data`], [`Toucan-Instrument`] & [`Toucan-Integration`] for
comprehensive documentation of other Toucan libraries.**

[`Toucan`]: https://github.com/brbtavares/toucan
[`Toucan-Data`]: https://github.com/brbtavares/toucan/tree/main/toucan-data
[`Toucan-Instrument`]: https://github.com/brbtavares/toucan/tree/main/toucan-instrument
[`Toucan-Integration`]: https://github.com/brbtavares/toucan/tree/main/toucan-integration
[toucan-examples]: https://github.com/brbtavares/toucan/tree/main/toucan/examples

## Overview

High-performance and normalised trading interface capable of executing across many financial venues. Also provides
a feature rich simulated exchange to assist with backtesting and dry-trading. Communicate with an exchange by 
initialising it's associated `ExecutionClient` instance. 

## Examples

* See [here][toucan-examples] for example of Toucan-Instrument in action.
* See other sub-crates for further examples of each library.
