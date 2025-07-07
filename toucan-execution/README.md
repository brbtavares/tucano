# Toucan-Execution
Stream private account data from financial venues, and execute (live or mock) orders. Also provides
a feature rich MockExchange and MockExecutionClient to assist with backtesting and paper-trading.

**It is:**
* **Easy**: ExecutionClient trait provides a unified and simple language for interacting with exchanges.
* **Normalised**: Allow your strategy to communicate with every real or MockExchange using the same interface.
* **Extensible**: Toucan-Execution is highly extensible, making it easy to contribute by adding new exchange integrations!

**See: [`Toucan`], [`Toucan-Data`], [`Toucan-Instrument`] & [`Toucan-Integration`] for
comprehensive documentation of other Toucan libraries.**

[![MIT licensed][mit-badge]][mit-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/brbtavares/toucan/blob/main/LICENSE

[API Documentation]

[`Toucan`]: https://github.com/brbtavares/toucan
[`Toucan-Data`]: https://github.com/brbtavares/toucan/tree/main/toucan-data
[`Toucan-Instrument`]: https://github.com/brbtavares/toucan/tree/main/toucan-instrument
[`Toucan-Integration`]: https://github.com/brbtavares/toucan/tree/main/toucan-integration
[toucan-examples]: https://github.com/brbtavares/toucan/tree/main/toucan/examples
[API Documentation]: https://docs.rs/toucan-execution/latest/toucan_execution

## Overview
High-performance and normalised trading interface capable of executing across many financial venues. Also provides
a feature rich simulated exchange to assist with backtesting and dry-trading. Communicate with an exchange by 
initialising it's associated `ExecutionClient` instance. 

## Examples
* See [here][toucan-examples] for example of Toucan-Instrument in action.
* See other sub-crates for further examples of each library.

## Getting Help
For questions, issues, or support with Toucan-Execution, please:
- Check the [API Documentation] for detailed usage examples
- Open an issue on the [GitHub repository](https://github.com/brbtavares/toucan/issues)
- Review the examples in the [toucan-examples] directory

## Support Development
If you find Toucan-Execution useful, consider:
- ⭐ Starring the repository
- 🐛 Reporting bugs or issues
- 🔧 Contributing new exchange integrations
- 📖 Improving documentation

## Contributing
Thanks for helping to develop the Toucan ecosystem! To contribute:
- Open issues for bugs or feature requests
- Submit pull requests for improvements
- Add new exchange execution clients
- Improve documentation and examples

### Licence
This project is licensed under the [MIT license].

[MIT license]: https://github.com/brbtavares/toucan/blob/main/LICENSE

### Contribution License Agreement

Any contribution you intentionally submit for inclusion in Toucan workspace crates shall be:
1. Licensed under MIT
2. Subject to all disclaimers and limitations of liability stated below
3. Provided without any additional terms or conditions
4. Submitted with the understanding that the educational-only purpose and risk warnings apply

By submitting a contribution, you certify that you have the right to do so under these terms.

## LEGAL DISCLAIMER AND LIMITATION OF LIABILITY

PLEASE READ THIS DISCLAIMER CAREFULLY BEFORE USING THE SOFTWARE. BY ACCESSING OR USING THE SOFTWARE, YOU ACKNOWLEDGE AND AGREE TO BE BOUND BY THE TERMS HEREIN.

1. EDUCATIONAL PURPOSE
   This software and related documentation ("Software") are provided solely for educational and research purposes. The Software is not intended, designed, tested, verified or certified for commercial deployment, live trading, or production use of any kind.

2. NO FINANCIAL ADVICE
   Nothing contained in the Software constitutes financial, investment, legal, or tax advice. No aspect of the Software should be relied upon for trading decisions or financial planning. Users are strongly advised to consult qualified professionals for investment guidance specific to their circumstances.

3. ASSUMPTION OF RISK
   Trading in financial markets, including but not limited to cryptocurrencies, securities, derivatives, and other financial instruments, carries substantial risk of loss. Users acknowledge that:
   a) They may lose their entire investment;
   b) Past performance does not indicate future results;
   c) Hypothetical or simulated performance results have inherent limitations and biases.

4. DISCLAIMER OF WARRANTIES
   THE SOFTWARE IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED. TO THE MAXIMUM EXTENT PERMITTED BY LAW, THE AUTHORS AND COPYRIGHT HOLDERS EXPRESSLY DISCLAIM ALL WARRANTIES, INCLUDING BUT NOT LIMITED TO:
   a) MERCHANTABILITY
   b) FITNESS FOR A PARTICULAR PURPOSE
   c) NON-INFRINGEMENT
   d) ACCURACY OR RELIABILITY OF RESULTS
   e) SYSTEM INTEGRATION
   f) QUIET ENJOYMENT

5. LIMITATION OF LIABILITY
   IN NO EVENT SHALL THE AUTHORS, COPYRIGHT HOLDERS, CONTRIBUTORS, OR ANY AFFILIATED PARTIES BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING BUT NOT LIMITED TO PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES, LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

6. REGULATORY COMPLIANCE
   The Software is not registered with, endorsed by, or approved by any financial regulatory authority. Users are solely responsible for:
   a) Determining whether their use complies with applicable laws and regulations
   b) Obtaining any required licenses, permits, or registrations
   c) Meeting any regulatory obligations in their jurisdiction

7. INDEMNIFICATION
   Users agree to indemnify, defend, and hold harmless the authors, copyright holders, and any affiliated parties from and against any claims, liabilities, damages, losses, and expenses arising from their use of the Software.

8. ACKNOWLEDGMENT
   BY USING THE SOFTWARE, USERS ACKNOWLEDGE THAT THEY HAVE READ THIS DISCLAIMER, UNDERSTOOD IT, AND AGREE TO BE BOUND BY ITS TERMS AND CONDITIONS.

THE ABOVE LIMITATIONS MAY NOT APPLY IN JURISDICTIONS THAT DO NOT ALLOW THE EXCLUSION OF CERTAIN WARRANTIES OR LIMITATIONS OF LIABILITY.
