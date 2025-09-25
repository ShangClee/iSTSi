import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { Badge } from './ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger } from './ui/alert-dialog';
import { Settings, RefreshCw, AlertTriangle, CheckCircle, XCircle, Clock, Bitcoin, Coins, ArrowRightLeft } from 'lucide-react';

const contractAddresses = {
  kycRegistry: '0x1234...5678',
  istsiToken: '0x2345...6789',
  fungibleToken: '0x3456...7890',
  reserveManager: '0x4567...8901',
};

const operationHistory = [
  {
    id: '0x1a2b3c4d',
    type: 'BitcoinDeposit',
    user: '0xuser1...1234',
    amount: '0.5 BTC',
    status: 'completed',
    timestamp: '2024-01-15 14:30:25',
    gasUsed: 125000,
  },
  {
    id: '0x2b3c4d5e',
    type: 'TokenWithdrawal',
    user: '0xuser2...5678',
    amount: '50000 iSTSi',
    status: 'pending',
    timestamp: '2024-01-15 14:28:12',
    gasUsed: 98000,
  },
  {
    id: '0x3c4d5e6f',
    type: 'CrossTokenExchange',
    user: '0xuser3...9012',
    amount: '10000 FT -> 25000 iSTSi',
    status: 'failed',
    timestamp: '2024-01-15 14:25:45',
    gasUsed: 156000,
  },
];

const pendingOperations = [
  {
    id: '0x5f6e7d8c',
    type: 'BitcoinDeposit',
    user: '0xuser4...3456',
    amount: '1.2 BTC',
    step: 'KYC Verification',
    progress: 65,
    estimatedCompletion: '5 minutes',
  },
  {
    id: '0x6e7d8c9b',
    type: 'TokenWithdrawal',
    user: '0xuser5...7890',
    amount: '75000 iSTSi',
    step: 'Reserve Validation',
    progress: 30,
    estimatedCompletion: '8 minutes',
  },
];

export function IntegrationRouter() {
  const [selectedOperation, setSelectedOperation] = useState('BitcoinDeposit');
  const [isUpdating, setIsUpdating] = useState(false);

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed': return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'pending': return <Clock className="w-4 h-4 text-yellow-500" />;
      case 'failed': return <XCircle className="w-4 h-4 text-red-500" />;
      default: return <Clock className="w-4 h-4 text-gray-500" />;
    }
  };

  const getOperationIcon = (type: string) => {
    switch (type) {
      case 'BitcoinDeposit': return <Bitcoin className="w-4 h-4" />;
      case 'TokenWithdrawal': return <Coins className="w-4 h-4" />;
      case 'CrossTokenExchange': return <ArrowRightLeft className="w-4 h-4" />;
      default: return <Settings className="w-4 h-4" />;
    }
  };

  const handleUpdateContracts = async () => {
    setIsUpdating(true);
    // Simulate contract update
    await new Promise(resolve => setTimeout(resolve, 2000));
    setIsUpdating(false);
  };

  return (
    <div className="space-y-6">
      <Tabs defaultValue="status" className="space-y-4">
        <TabsList>
          <TabsTrigger value="status">Router Status</TabsTrigger>
          <TabsTrigger value="operations">Operations</TabsTrigger>
          <TabsTrigger value="config">Configuration</TabsTrigger>
        </TabsList>

        <TabsContent value="status" className="space-y-4">
          {/* Router Overview */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Settings className="w-5 h-5" />
                <span>Integration Router Status</span>
              </CardTitle>
              <CardDescription>
                Central orchestrator for all cross-contract operations
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                <div className="text-center p-4 border rounded-lg">
                  <div className="text-2xl font-bold text-green-600">Active</div>
                  <div className="text-sm text-muted-foreground">Router Status</div>
                </div>
                <div className="text-center p-4 border rounded-lg">
                  <div className="text-2xl font-bold">1,247</div>
                  <div className="text-sm text-muted-foreground">Operations Today</div>
                </div>
                <div className="text-center p-4 border rounded-lg">
                  <div className="text-2xl font-bold">99.8%</div>
                  <div className="text-sm text-muted-foreground">Success Rate</div>
                </div>
                <div className="text-center p-4 border rounded-lg">
                  <div className="text-2xl font-bold">2.3s</div>
                  <div className="text-sm text-muted-foreground">Avg Processing Time</div>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Contract Connections */}
          <Card>
            <CardHeader>
              <CardTitle>Connected Contracts</CardTitle>
              <CardDescription>Status of all integrated smart contracts</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {Object.entries(contractAddresses).map(([name, address]) => (
                  <div key={name} className="flex items-center justify-between p-3 border rounded-lg">
                    <div className="flex items-center space-x-3">
                      <CheckCircle className="w-5 h-5 text-green-500" />
                      <div>
                        <p className="font-medium capitalize">{name.replace(/([A-Z])/g, ' $1').trim()}</p>
                        <p className="text-sm text-muted-foreground font-mono">{address}</p>
                      </div>
                    </div>
                    <Badge variant="default">Connected</Badge>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Pending Operations */}
          <Card>
            <CardHeader>
              <CardTitle>Pending Operations</CardTitle>
              <CardDescription>Operations currently being processed</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {pendingOperations.map((op) => (
                  <div key={op.id} className="p-4 border rounded-lg">
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center space-x-2">
                        {getOperationIcon(op.type)}
                        <span className="font-medium">{op.type}</span>
                        <Badge variant="outline">{op.id}</Badge>
                      </div>
                      <span className="text-sm text-muted-foreground">{op.estimatedCompletion} remaining</span>
                    </div>
                    <div className="space-y-2">
                      <div className="flex justify-between text-sm">
                        <span>User: {op.user}</span>
                        <span>Amount: {op.amount}</span>
                      </div>
                      <div className="flex justify-between text-sm">
                        <span>Current Step: {op.step}</span>
                        <span>{op.progress}% complete</span>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-2 dark:bg-gray-700">
                        <div 
                          className="bg-blue-600 h-2 rounded-full transition-all duration-500" 
                          style={{ width: `${op.progress}%` }}
                        ></div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="operations" className="space-y-4">
          {/* Manual Operation Execution */}
          <Card>
            <CardHeader>
              <CardTitle>Execute Operation</CardTitle>
              <CardDescription>Manually trigger integration operations</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="operation-type">Operation Type</Label>
                  <Select value={selectedOperation} onValueChange={setSelectedOperation}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="BitcoinDeposit">Bitcoin Deposit</SelectItem>
                      <SelectItem value="TokenWithdrawal">Token Withdrawal</SelectItem>
                      <SelectItem value="CrossTokenExchange">Cross Token Exchange</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="user-address">User Address</Label>
                  <Input id="user-address" placeholder="0x..." />
                </div>
              </div>
              
              {selectedOperation === 'BitcoinDeposit' && (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="btc-amount">BTC Amount</Label>
                    <Input id="btc-amount" placeholder="0.00000000" />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="tx-hash">Transaction Hash</Label>
                    <Input id="tx-hash" placeholder="Bitcoin transaction hash" />
                  </div>
                </div>
              )}

              {selectedOperation === 'TokenWithdrawal' && (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="token-amount">Token Amount</Label>
                    <Input id="token-amount" placeholder="0" />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="btc-address">BTC Address</Label>
                    <Input id="btc-address" placeholder="Bitcoin address" />
                  </div>
                </div>
              )}

              {selectedOperation === 'CrossTokenExchange' && (
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="from-token">From Token</Label>
                    <Select>
                      <SelectTrigger>
                        <SelectValue placeholder="Select token" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="istsi">iSTSi Token</SelectItem>
                        <SelectItem value="fungible">Fungible Token</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="to-token">To Token</Label>
                    <Select>
                      <SelectTrigger>
                        <SelectValue placeholder="Select token" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="istsi">iSTSi Token</SelectItem>
                        <SelectItem value="fungible">Fungible Token</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="exchange-amount">Amount</Label>
                    <Input id="exchange-amount" placeholder="0" />
                  </div>
                </div>
              )}

              <AlertDialog>
                <AlertDialogTrigger asChild>
                  <Button className="w-full">Execute Operation</Button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>Confirm Operation</AlertDialogTitle>
                    <AlertDialogDescription>
                      Are you sure you want to execute this {selectedOperation}? This action cannot be undone.
                    </AlertDialogDescription>
                  </AlertDialogHeader>
                  <AlertDialogFooter>
                    <AlertDialogCancel>Cancel</AlertDialogCancel>
                    <AlertDialogAction>Execute</AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            </CardContent>
          </Card>

          {/* Operation History */}
          <Card>
            <CardHeader>
              <CardTitle>Recent Operations</CardTitle>
              <CardDescription>History of executed operations</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {operationHistory.map((op) => (
                  <div key={op.id} className="flex items-center justify-between p-3 border rounded-lg">
                    <div className="flex items-center space-x-3">
                      {getStatusIcon(op.status)}
                      {getOperationIcon(op.type)}
                      <div>
                        <p className="font-medium">{op.type}</p>
                        <p className="text-sm text-muted-foreground">
                          {op.user} • {op.amount}
                        </p>
                      </div>
                    </div>
                    <div className="text-right">
                      <Badge variant="outline">{op.id}</Badge>
                      <p className="text-sm text-muted-foreground mt-1">
                        {op.timestamp} • Gas: {op.gasUsed.toLocaleString()}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="config" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Router Configuration</CardTitle>
              <CardDescription>Update contract addresses and router settings</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {Object.entries(contractAddresses).map(([name, address]) => (
                  <div key={name} className="space-y-2">
                    <Label htmlFor={name} className="capitalize">
                      {name.replace(/([A-Z])/g, ' $1').trim()} Address
                    </Label>
                    <Input id={name} value={address} readOnly />
                  </div>
                ))}
              </div>
              
              <div className="flex space-x-4 pt-4">
                <Button 
                  onClick={handleUpdateContracts}
                  disabled={isUpdating}
                  className="flex items-center space-x-2"
                >
                  <RefreshCw className={`w-4 h-4 ${isUpdating ? 'animate-spin' : ''}`} />
                  <span>{isUpdating ? 'Updating...' : 'Update Contracts'}</span>
                </Button>
                
                <AlertDialog>
                  <AlertDialogTrigger asChild>
                    <Button variant="destructive" className="flex items-center space-x-2">
                      <AlertTriangle className="w-4 h-4" />
                      <span>Emergency Pause</span>
                    </Button>
                  </AlertDialogTrigger>
                  <AlertDialogContent>
                    <AlertDialogHeader>
                      <AlertDialogTitle>Emergency Pause</AlertDialogTitle>
                      <AlertDialogDescription>
                        This will immediately halt all router operations. Only use in emergency situations.
                      </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                      <AlertDialogCancel>Cancel</AlertDialogCancel>
                      <AlertDialogAction>Pause Router</AlertDialogAction>
                    </AlertDialogFooter>
                  </AlertDialogContent>
                </AlertDialog>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}