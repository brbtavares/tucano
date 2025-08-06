# Bibliotecas Nativas

Este diretório contém as bibliotecas nativas necessárias para o wrapper ProfitDLL.

## Arquivos

### Windows
- `ProfitDLL.dll` - Biblioteca principal da ProfitDLL para Windows
- `ProfitDLL.lib` - Biblioteca de import (se disponível)

### Instalação

1. **Baixe a ProfitDLL oficial** do provedor
2. **Copie os arquivos** para este diretório:
   ```bash
   cp /caminho/para/ProfitDLL.dll profit-dll/lib/
   ```

3. **Configure o PATH** (opcional):
   ```bash
   # Windows
   set PATH=%PATH%;C:\caminho\para\toucan\profit-dll\lib
   
   # PowerShell
   $env:PATH += ";C:\caminho\para\toucan\profit-dll\lib"
   ```

## Uso no Código

O wrapper procura a DLL automaticamente em:

1. **Caminho especificado** no `ProfitConnector::new(Some("caminho"))`
2. **Diretório `lib/`** relativo ao projeto
3. **PATH do sistema**

### Exemplo
```rust
// Usa DLL no diretório lib/
let connector = ProfitConnector::new(None)?;

// Ou caminho específico
let connector = ProfitConnector::new(Some("./lib/ProfitDLL.dll"))?;
```

## Notas

- ⚠️ **Não commitar** arquivos DLL no Git (são binários grandes)
- ✅ **Documentar** versão e fonte da DLL
- 🔒 **Verificar** integridade dos arquivos baixados

## Licença

As bibliotecas nativas são propriedade de seus respectivos fornecedores.
Consulte a documentação oficial para termos de uso.
