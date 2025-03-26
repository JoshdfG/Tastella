# Configuration (override these when running make)
WALLET ?= xion1quzuep30mjz9vfu6mevgy4tcempf3sh9ag3zwk
CHAIN_ID ?= xion-testnet-2
RPC_NODE ?= https://rpc.xion-testnet-2.burnt.com:443
GAS_PRICES ?= 0.1uxion
LABEL ?= tastella
FEE_ADDRESS ?= xion1c6vhzppnktpnw0l80kcgeqysa9wms36gxqnqf4
CONTRACT ?= xion1sljvzt7clk2l5aytf8ur5sllafaphd8v2ed435kv85mjdncts4tqc8hfr7
MIGRATE_MSG ?= {"convert_to_multi_owner":{}}

.PHONY: all compile store code-id instantiate contract-addr deploy optimize upgrade-store upgrade verify clean help

all: deploy

# Shared: Compile and optimize WASM
compile:
	@echo "\033[34mCompiling and optimizing WASM...\033[0m"
	docker run --rm -v "$(PWD)":/code \
		--mount type=volume,source="$(shell basename "$(PWD)")_cache",target=/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		cosmwasm/optimizer:0.16.0

# Deploy: Store contract on chain
store: compile
	@echo "\033[34mStoring contract...\033[0m"
	@RES=$$(xiond tx wasm store ./artifacts/tastella.wasm \
		--chain-id $(CHAIN_ID) \
		--gas-adjustment 1.3 \
		--gas-prices $(GAS_PRICES) \
		--gas auto \
		-y --output json \
		--node $(RPC_NODE) \
		--from $(WALLET)) ; \
	TXHASH=$$(echo "$$RES" | jq -r '.txhash') ; \
	if [ -z "$$TXHASH" ] || [ "$$TXHASH" = "null" ]; then \
		echo "Error: Failed to get txhash from wasm store"; \
		exit 1; \
	fi; \
	echo "Transaction hash: \033[32m$$TXHASH\033[0m" ; \
	echo "$$TXHASH" > .txhash

# Deploy: Get CODE_ID
code-id: store
	@sleep 10
	@echo "\033[34mFetching CODE_ID...\033[0m"
	@TXHASH=$$(cat .txhash) ; \
	CODE_ID=$$(xiond query tx $$TXHASH --node $(RPC_NODE) --output json | jq -r '.events[] | select(.type == "store_code") | .attributes[] | select(.key == "code_id") | .value') ; \
	if [ -z "$$CODE_ID" ] || [ "$$CODE_ID" = "null" ]; then \
		echo "Error: Failed to get CODE_ID from tx $$TXHASH"; \
		exit 1; \
	fi; \
	echo "CODE_ID: \033[32m$$CODE_ID\033[0m" ; \
	echo "$$CODE_ID" > .codeid

# Deploy: Instantiate contract
instantiate: code-id
	@echo "\033[34mInstantiating contract...\033[0m"
	@CODE_ID=$$(cat .codeid) ; \
	MSG='{ \
		"platform_name": "Tastella", \
		"platform_description": "A decentralized food delivery platform", \
		"owner_address": "$(WALLET)", \
		"fee_percentage": "0.05", \
		"fee_address": "$(FEE_ADDRESS)" \
	}' ; \
	RES=$$(xiond tx wasm instantiate $$CODE_ID "$$MSG" \
		--from $(WALLET) \
		--label $(LABEL) \
		--gas-prices $(GAS_PRICES) \
		--gas auto \
		--gas-adjustment 1.3 \
		--admin $(WALLET) \
		--chain-id $(CHAIN_ID) \
		--node $(RPC_NODE) \
		-y --output json) ; \
	TXHASH=$$(echo "$$RES" | jq -r '.txhash') ; \
	if [ -z "$$TXHASH" ] || [ "$$TXHASH" = "null" ]; then \
		echo "Error: Failed to get txhash from instantiate"; \
		exit 1; \
	fi; \
	echo "Instantiation tx hash: \033[32m$$TXHASH\033[0m" ; \
	echo "$$TXHASH" > .instantiate_txhash

# Deploy: Get contract address
contract-addr: instantiate
	@sleep 10
	@echo "\033[34mFetching contract address...\033[0m"
	@TXHASH=$$(cat .instantiate_txhash) ; \
	CONTRACT=$$(xiond query tx $$TXHASH --node $(RPC_NODE) --output json | \
		jq -r '.events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value') ; \
	if [ -z "$$CONTRACT" ] || [ "$$CONTRACT" = "null" ]; then \
		echo "Error: Failed to get contract address from tx $$TXHASH"; \
		exit 1; \
	fi; \
	echo "\033[32mContract successfully deployed!\033[0m" ; \
	echo "Contract address: \033[36m$$CONTRACT\033[0m" ; \
	echo "$$CONTRACT" > .contract

# Deploy target
deploy: contract-addr

# Upgrade: Store new wasm
upgrade-store: compile
	@echo "Storing tastella.wasm on Xion testnet for upgrade..."
	@RES=$$(xiond tx wasm store ./artifacts/tastella.wasm \
		--chain-id "$(CHAIN_ID)" \
		--gas-adjustment 1.3 \
		--gas-prices 0.1uxion \
		--gas auto \
		-y --output json \
		--node "$(RPC_NODE)" \
		--from "$(WALLET)"); \
	TXHASH=$$(echo "$$RES" | jq -r '.txhash'); \
	if [ -z "$$TXHASH" ] || [ "$$TXHASH" = "null" ]; then \
		echo "Error: Failed to get txhash from wasm store"; \
		exit 1; \
	fi; \
	echo "Wasm store txhash: $$TXHASH"; \
	echo "$$TXHASH" > .upgrade_txhash.tmp; \
	NEW_CODE_ID=$$(xiond query tx "$$TXHASH" --node "$(RPC_NODE)" -o json | jq -r '.events[] | select(.type == "store_code") | .attributes[] | select(.key == "code_id") | .value'); \
	if [ -z "$$NEW_CODE_ID" ] || [ "$$NEW_CODE_ID" = "null" ]; then \
		echo "Error: Failed to get NEW_CODE_ID from tx $$TXHASH"; \
		exit 1; \
	fi; \
	echo "New CODE_ID: $$NEW_CODE_ID"; \
	echo "$$NEW_CODE_ID" > .upgrade_code_id.tmp

# Upgrade: Migrate the contract
upgrade: upgrade-store
	@echo "Migrating contract $(CONTRACT) to code ID $$(cat .upgrade_code_id.tmp)..."
	@MIGRATE_RES=$$(xiond tx wasm migrate "$(CONTRACT)" "$$(cat .upgrade_code_id.tmp)" '$(MIGRATE_MSG)' \
		--from "$(WALLET)" \
		--gas-prices 0.025uxion \
		--gas auto \
		--gas-adjustment 1.3 \
		-y \
		--chain-id "$(CHAIN_ID)" \
		--node "$(RPC_NODE)" --output json); \
	MIGRATE_TXHASH=$$(echo "$$MIGRATE_RES" | jq -r '.txhash'); \
	if [ -z "$$MIGRATE_TXHASH" ] || [ "$$MIGRATE_TXHASH" = "null" ]; then \
		echo "Error: Failed to get txhash from migrate"; \
		exit 1; \
	fi; \
	echo "Migrate txhash: $$MIGRATE_TXHASH"; \
	echo "$$MIGRATE_TXHASH" > .upgrade_migrate_txhash.tmp

# Verify the upgrade
verify: upgrade
	@echo "Verifying upgrade..."
	@xiond query tx "$$(cat .upgrade_migrate_txhash.tmp)" --node "$(RPC_NODE)" -o json

# Clean temporary files
clean:
	@rm -f .txhash .codeid .instantiate_txhash .contract .upgrade_txhash.tmp .upgrade_code_id.tmp .upgrade_migrate_txhash.tmp

# Help
help:
	@echo "Available targets:"
	@echo "  all      : Run deploy (default)"
	@echo "  deploy   : Full deployment pipeline (compile, store, instantiate)"
	@echo "  upgrade  : Upgrade existing contract (compile, store, migrate)"
	@echo "  verify   : Verify the upgrade"
	@echo "  clean    : Remove temporary files"
	@echo "  help     : Show this help message"