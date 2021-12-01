import React from 'react'
import { StakingComponent } from '../../components';
import {
    WalletModalProvider,
    WalletDisconnectButton,
    WalletMultiButton,
} from '@solana/wallet-adapter-react-ui'
import { useWallet } from '@solana/wallet-adapter-react'

export default function Home() {    
    const wallet = useWallet()
    const { publicKey } = wallet;
    console.log("Connected wallet", publicKey)
    
    return (
        <div>
            Staking Pool
            <WalletModalProvider>
                {
                    publicKey ? 
                        <div>
                            <StakingComponent /> 
                            <WalletDisconnectButton /> 
                        </div>
                    :
                        <WalletMultiButton />
                        
                }
            </WalletModalProvider>                
        </div>
    )
}
