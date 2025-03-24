import './NetworkSelector.css';

type Network = 'testnet' | 'mainnet';

interface NetworkSelectorProps {
  onNetworkChange: (network: Network) => void;
  currentNetwork: Network;
}

export function NetworkSelector({ onNetworkChange, currentNetwork }: NetworkSelectorProps) {
  return (
    <div className="network-selector">
      <select
        value={currentNetwork}
        onChange={(e) => onNetworkChange(e.target.value as Network)}
        className="network-select"
      >
        <option value="testnet">Testnet</option>
        <option value="mainnet">Mainnet</option>
      </select>
    </div>
  );
}