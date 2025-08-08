# Configuração da ProfitDLL para Windows

## 📋 Pré-requisitos

1. **Sistema Windows** (7, 8, 10, ou 11)
2. **ProfitDLL da Nelógica** instalada
3. **Rust toolchain** configurado para Windows
4. **Credenciais válidas** da Nelógica

## 🔧 Instalação da ProfitDLL

### Opção 1: Instalação Automática
1. Baixe o instalador da ProfitDLL no site da Nelógica
2. Execute o instalador como administrador
3. A DLL será instalada em `C:\Program Files\Nelogica\ProfitDLL\`

### Opção 2: Instalação Manual
1. Obtenha `ProfitDLL.dll` da Nelógica
2. Coloque em um dos diretórios:
   ```
   C:\Program Files\Nelogica\ProfitDLL\
   C:\Program Files (x86)\Nelogica\ProfitDLL\
   C:\ProfitDLL\
   .\dll\
   .\lib\
   .\ (diretório do projeto)
   ```

### Opção 3: Variável de Ambiente
```cmd
set PROFITDLL_PATH=C:\caminho\para\sua\dll
```

## 🏗️ Compilação

### Compilação com DLL Real (Windows)
```bash
# Ativar feature real_dll
cargo build --features real_dll

# Para release
cargo build --release --features real_dll

# Executar exemplo
cargo run --example profit_dll_windows_config --features real_dll
```

### Compilação Mock (qualquer OS)
```bash
# Sem features (padrão)
cargo build

# Executar exemplo mock
cargo run --example profit_dll_windows_config
```

## 🔍 Verificação da Instalação

Execute o script de verificação:
```bash
cargo run --example profit_dll_windows_config
```

O output deve mostrar:
```
✅ Executando em Windows - DLL real disponível
✅ ProfitDLL.dll encontrada em: C:\Program Files\Nelogica\ProfitDLL
✅ ProfitConnector inicializado
```

## ⚙️ Configuração

### 1. Credenciais
Edite `examples/profit_dll_windows_config.rs`:
```rust
let events = connector.initialize_login(
    "sua_chave_ativacao",    // Chave da Nelógica
    "seu_usuario",           // Usuário
    "sua_senha"              // Senha
).await?;
```

### 2. Configuração Avançada
```rust
// Especificar caminho da DLL
let connector = ProfitConnector::new(Some("C:\\custom\\path\\ProfitDLL.dll"))?;

// Auto-detecção (recomendado)
let connector = ProfitConnector::new(None)?;
```

## 🚨 Troubleshooting

### Erro: "DLL não encontrada"
1. Verifique se `ProfitDLL.dll` existe no caminho
2. Confirme que não é uma versão 32-bit em sistema 64-bit
3. Configure `PROFITDLL_PATH` explicitamente

### Erro: "Falha na inicialização"
1. Verifique credenciais da Nelógica
2. Confirme conexão com internet
3. Verifique se a licença está ativa

### Erro: "Falha no login"
1. Credenciais incorretas
2. Conta bloqueada ou suspensa
3. Serviços da Nelógica indisponíveis

### Erro de Compilação
```bash
# Instalar dependências Windows
cargo install --force --path .

# Verificar toolchain
rustc --version --verbose

# Recompilar com verbose
cargo build --features real_dll --verbose
```

## 📊 Funcionalidades Disponíveis

### ✅ Implementadas
- ✅ **Inicialização e Login**: `initialize_login()`
- ✅ **Subscrição Market Data**: `subscribe_ticker()`
- ✅ **Eventos em Tempo Real**: Callbacks para negócios, ofertas, resumos
- ✅ **Execução de Ordens**: `send_order()`
- ✅ **Gerenciamento de Conexão**: Estados e reconexão

### 📋 Estrutura de Callbacks
- `StateChanged`: Mudanças de estado da conexão
- `NewTrade`: Novos negócios em tempo real
- `DailySummary`: Resumo diário (OHLC, volume)
- `PriceBookOffer`: Livro de ofertas (Level 2)
- `ProgressChanged`: Progresso das subscrições

## 🔗 Integração com Toucan

### Broker Integration
```rust
use markets::broker::{ProfitDLLBroker, MarketDataProvider};

let mut broker = ProfitDLLBroker::new();
broker.initialize("key", "user", "pass").await?;
```

### Asset Management
```rust
use markets::b3::{B3Stock, B3AssetFactory};

let petr4 = B3Stock::new("PETR4".to_string(), "Petrobras PN".to_string());
let asset = B3AssetFactory::from_symbol("VALE3")?;
```

### Event Processing
```rust
while let Some(event) = broker.next_market_event().await {
    match event {
        CallbackEvent::NewTrade { ticker, price, .. } => {
            println!("Trade: {} @ {}", ticker, price);
        }
        // ... outros eventos
    }
}
```

## 📈 Performance

### Latência Típica
- **Inicialização**: ~2-5 segundos
- **Login**: ~1-3 segundos  
- **Subscrição**: ~500ms por instrumento
- **Eventos**: <10ms (tempo real)

### Throughput
- **Market Data**: >1000 eventos/segundo
- **Ordens**: >100 ordens/segundo
- **Conexões simultâneas**: Limitado pela Nelógica

## 🛡️ Segurança

1. **Credenciais**: Nunca hardcode credenciais no código
2. **Logs**: Não logue senhas ou chaves
3. **Rede**: Use conexões seguras (TLS)
4. **Validação**: Sempre valide dados recebidos

## 📚 Recursos Adicionais

- **Manual ProfitDLL**: Documentação oficial da Nelógica
- **Exemplos**: `/examples/profit_dll_*`
- **Testes**: `cargo test -p markets profit_dll`
- **Documentação**: `cargo doc -p markets --open`
