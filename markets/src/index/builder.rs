// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.


use crate::{
	asset::{Asset, AssetIndex, ExchangeAsset},
	exchange::{ExchangeId, ExchangeIndex},
	index::{
		find_asset_by_exchange_and_name_internal, find_exchange_by_exchange_id, IndexedInstruments,
	},
	instrument::{spec::OrderQuantityUnits, Instrument, InstrumentIndex},
	Keyed,
};

#[derive(Debug, Default)]
pub struct IndexedInstrumentsBuilder {
	exchanges: Vec<ExchangeId>,
	instruments: Vec<Instrument<ExchangeId, Asset>>, // ExchangeId keyed instrument (later remapped to ExchangeIndex)
	assets: Vec<ExchangeAsset<Asset>>,               // ExchangeId keyed assets   (later remapped to AssetIndex)
}

impl IndexedInstrumentsBuilder {
	pub fn new() -> Self { Self::default() }

	pub fn add_instrument(mut self, instrument: Instrument<ExchangeId, Asset>) -> Self {
		// Track exchange
		self.exchanges.push(instrument.exchange);

		// Track underlying base & quote assets
		self.assets.push(ExchangeAsset::new(instrument.exchange, instrument.underlying.base.clone()));
		self.assets.push(ExchangeAsset::new(instrument.exchange, instrument.underlying.quote.clone()));

		// Settlement asset (if derivative)
		if let Some(settlement_asset) = instrument.kind.settlement_asset() {
			self.assets.push(ExchangeAsset::new(instrument.exchange, settlement_asset.clone()));
		}

		// Asset-based quantity unit (if present)
		if let Some(spec) = instrument.spec.as_ref() {
			if let OrderQuantityUnits::Asset(asset) = &spec.quantity.unit {
				self.assets.push(ExchangeAsset::new(instrument.exchange, asset.clone()));
			}
		}

		self.instruments.push(instrument);
		self
	}

	pub fn build(mut self) -> IndexedInstruments {
		// Normalize & deduplicate
		self.exchanges.sort();
		self.exchanges.dedup();
		self.instruments.sort();
		self.instruments.dedup();
		self.assets.sort();
		self.assets.dedup();

		// Index exchanges
		let exchanges = self
			.exchanges
			.into_iter()
			.enumerate()
			.map(|(i, exchange)| Keyed::new(ExchangeIndex::new(i), exchange))
			.collect::<Vec<_>>();

		// Index assets
		let assets = self
			.assets
			.into_iter()
			.enumerate()
			.map(|(i, exchange_asset)| Keyed::new(AssetIndex::new(i), exchange_asset))
			.collect::<Vec<_>>();

		// Index instruments and remap embedded asset & exchange keys
		let instruments = self
			.instruments
			.into_iter()
			.enumerate()
			.map(|(i, instrument)| {
				let exchange_id = instrument.exchange;
				let exchange_key = find_exchange_by_exchange_id(&exchanges, &exchange_id)
					.expect("every exchange related to every instrument has been added");
				let instrument = instrument.map_exchange_key(Keyed::new(exchange_key, exchange_id));
				let instrument = instrument
					.map_asset_key_with_lookup(|asset: &Asset| {
						find_asset_by_exchange_and_name_internal(
							&assets,
							exchange_id,
							&asset.name_internal,
						)
					})
					.expect("every asset related to every instrument has been added");
				Keyed::new(InstrumentIndex::new(i), instrument)
			})
			.collect();

		IndexedInstruments { exchanges, instruments, assets }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		instrument::{
			kind::InstrumentKind,
			name::{InstrumentNameExchange, InstrumentNameInternal},
			quote::InstrumentQuoteAsset,
			spec::{
				InstrumentSpec, InstrumentSpecNotional, InstrumentSpecPrice, InstrumentSpecQuantity,
			},
		},
		test_utils::{exchange_asset, instrument},
		Underlying,
	};
	use rust_decimal_macros::dec;

	#[test]
	fn test_builder_basic_spot() {
		let indexed = IndexedInstrumentsBuilder::default()
			.add_instrument(instrument(ExchangeId::BinanceSpot, "btc", "usdt"))
			.build();

		assert_eq!(indexed.exchanges().len(), 1);
		assert_eq!(indexed.assets().len(), 2);
		assert_eq!(indexed.instruments().len(), 1);
		assert_eq!(indexed.exchanges()[0].value, ExchangeId::BinanceSpot);
		assert_eq!(indexed.assets()[0].value, exchange_asset(ExchangeId::BinanceSpot, "btc"));
		assert_eq!(indexed.assets()[1].value, exchange_asset(ExchangeId::BinanceSpot, "usdt"));
		assert_eq!(indexed.instruments()[0].value.exchange.value, ExchangeId::BinanceSpot);
	}

	#[test]
	fn test_builder_deduplication() {
		let indexed = IndexedInstrumentsBuilder::default()
			.add_instrument(instrument(ExchangeId::BinanceSpot, "BTC", "USDT"))
			.add_instrument(instrument(ExchangeId::BinanceSpot, "BTC", "USDT"))
			.build();

		assert_eq!(indexed.exchanges().len(), 1);
		assert_eq!(indexed.assets().len(), 2);
		assert_eq!(indexed.instruments().len(), 1);
	}

	#[test]
	fn test_builder_multiple_exchanges() {
		let indexed = IndexedInstrumentsBuilder::default()
			.add_instrument(instrument(ExchangeId::BinanceSpot, "BTC", "USDT"))
			.add_instrument(instrument(ExchangeId::B3, "PETR4", "BRL"))
			.build();

		assert_eq!(indexed.exchanges().len(), 2);
		assert_eq!(indexed.assets().len(), 4);
		assert_eq!(indexed.instruments().len(), 2);
	}

	#[test]
	fn test_builder_asset_unit_handling() {
		let base_asset = Asset::new_from_exchange("BTC");
		let quote_asset = Asset::new_from_exchange("USDT");

		let instrument = Instrument::new(
			ExchangeId::BinanceSpot,
			"binance_spot_btc_usdt",
			"BTC-USDT",
			Underlying::new(base_asset.clone(), quote_asset.clone()),
			InstrumentQuoteAsset::UnderlyingQuote,
			InstrumentKind::Spot,
			Some(InstrumentSpec {
				price: InstrumentSpecPrice { min: dec!(0.1), tick_size: dec!(0.1) },
				quantity: InstrumentSpecQuantity {
					unit: OrderQuantityUnits::Asset(base_asset.clone()),
					min: dec!(0.001),
					increment: dec!(0.001),
				},
				notional: InstrumentSpecNotional { min: dec!(10) },
			}),
		);

		let indexed = IndexedInstrumentsBuilder::default().add_instrument(instrument).build();

		assert_eq!(indexed.assets().len(), 2);
		assert_eq!(indexed.assets()[0].value, exchange_asset(ExchangeId::BinanceSpot, "BTC"));
	}

	#[test]
	fn test_builder_ordering() {
		let indexed = IndexedInstrumentsBuilder::default()
			.add_instrument(instrument(ExchangeId::BinanceSpot, "BTC", "USDT"))
			.add_instrument(instrument(ExchangeId::B3, "VALE3", "BRL"))
			.build();

		assert_eq!(indexed.exchanges()[0].value, ExchangeId::BinanceSpot);
		assert_eq!(indexed.exchanges()[1].value, ExchangeId::B3);
		assert_eq!(indexed.assets()[0].value, exchange_asset(ExchangeId::BinanceSpot, "BTC"));
		assert_eq!(indexed.assets()[1].value, exchange_asset(ExchangeId::BinanceSpot, "USDT"));
		assert_eq!(indexed.assets()[2].value, exchange_asset(ExchangeId::B3, "VALE3"));
		assert_eq!(indexed.assets()[3].value, exchange_asset(ExchangeId::B3, "BRL"));
		assert_eq!(indexed.instruments()[0].value.exchange.value, ExchangeId::BinanceSpot);
		assert_eq!(indexed.instruments()[1].value.exchange.value, ExchangeId::B3);
	}
}

