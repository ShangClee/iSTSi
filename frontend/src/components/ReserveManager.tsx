import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { Badge } from './ui/badge';
import { Progress } from './ui/progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger } from './ui/alert-dialog';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell } from 'recharts';
import { Bitcoin, Shield, AlertTriangle, TrendingUp, TrendingDown, CheckCircle, Clock, RefreshCw, Wallet } from 'lucide-react';

interface ReserveManagerProps {
  systemState: {
    reserveRatio: number;
  };
}

const bitcoinHoldings = [
  { address: '1A1zP1...', type: 'Cold Storage', amount: 35.67891234, percentage: 78.1 },
  { address: '3J98t1...', type: 'Hot Wallet', amount: 5.12345678, percentage: 11.2 },
  { address: 'bc1qxy...', type: 'Multisig', amount: 4.87654322, percentage: 10.7 },
];

const recentTransactions = [
  {
    id: '1',
    type: 'deposit',
    txHash: '7d2b3f4a...',
    amount: 1.25000000,
    confirmations: 6,
    status: 'confirmed',
    timestamp: '2024-01-15 14:32:15',
  },
  {
    id: '2',
    type: 'withdrawal',
    txHash: '8e3c4f5b...',
    amount: 0.75000000,
    confirmations: 3,
    status: 'pending',
    timestamp: '2024-01-15 14:28:45',
  },
  {
    id: '3',
    type: 'deposit',
    txHash: '9f4d5e6c...',
    amount: 2.50000000,
    confirmations: 12,
    status: 'confirmed',
    timestamp: '2024-01-15 14:15:30',
  },
];

const reserveHistory = [
  { date: '2024-01-10', ratio: 103.2, btc: 44.2, tokens: 4298765432 },
  { date: '2024-01-11', ratio: 102.8, btc: 44.8, tokens: 4356789012 },
  { date: '2024-01-12', ratio: 103.1, btc: 45.1, tokens: 4378901234 },
  { date: '2024-01-13', ratio: 102.5, btc: 45.3, tokens: 4421012345 },
  { date: '2024-01-14', ratio: 103.0, btc: 45.5, tokens: 4445123456 },
  { date: '2024-01-15', ratio: 102.3, btc: 45.7, tokens: 4467891234 },
];

const pieData = [
  { name: 'Cold Storage', value: 78.1, color: '#8884d8' },
  { name: 'Hot Wallet', value: 11.2, color: '#82ca9d' },
  { name: 'Multisig', value: 10.7, color: '#ffc658' },
];

export function ReserveManager({ systemState }: ReserveManagerProps) {
  const [isGeneratingProof, setIsGeneratingProof] = useState(false);

  const totalBTC = bitcoinHoldings.reduce((sum, holding) => sum + holding.amount, 0);
  
  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'confirmed': return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'pending': return <Clock className="w-4 h-4 text-yellow-500" />;
      default: return <Clock className="w-4 h-4 text-gray-500" />;
    }
  };

  const getTransactionIcon = (type: string) => {
    return type === 'deposit' ? 
      <TrendingUp className="w-4 h-4 text-green-500" /> : 
      <TrendingDown className="w-4 h-4 text-red-500" />;
  };

  const handleGenerateProof = async () => {
    setIsGeneratingProof(true);
    // Simulate proof generation
    await new Promise(resolve => setTimeout(resolve, 3000));
    setIsGeneratingProof(false);
  };

  const getReserveStatus = (ratio: number) => {
    if (ratio >= 105) return { status: 'excellent', color: 'text-green-600', bg: 'bg-green-100' };
    if (ratio >= 102) return { status: 'good', color: 'text-green-600', bg: 'bg-green-100' };
    if (ratio >= 100) return { status: 'adequate', color: 'text-yellow-600', bg: 'bg-yellow-100' };
    return { status: 'critical', color: 'text-red-600', bg: 'bg-red-100' };
  };

  const reserveStatus = getReserveStatus(systemState.reserveRatio);

  return (
    <div className="space-y-6">
      <Tabs defaultValue="overview" className="space-y-4">
        <TabsList>
          <TabsTrigger value="overview">Reserve Overview</TabsTrigger>
          <TabsTrigger value="holdings">Bitcoin Holdings</TabsTrigger>
          <TabsTrigger value="transactions">Transactions</TabsTrigger>
          <TabsTrigger value="proof">Proof of Reserves</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="space-y-4">
          {/* Reserve Status */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Reserve Ratio</CardTitle>
                <Shield className="w-4 h-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{systemState.reserveRatio.toFixed(1)}%</div>
                <div className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium mt-2 ${reserveStatus.bg} ${reserveStatus.color}`}>
                  {reserveStatus.status.toUpperCase()}
                </div>
                <Progress value={Math.min(systemState.reserveRatio, 110)} className="mt-2" />
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Total BTC Reserves</CardTitle>
                <Bitcoin className="w-4 h-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{totalBTC.toFixed(8)}</div>
                <p className="text-xs text-muted-foreground">
                  ≈ ${(totalBTC * 40000).toLocaleString()}
                </p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Security Level</CardTitle>
                <Wallet className="w-4 h-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-green-600">95.2%</div>
                <p className="text-xs text-muted-foreground">
                  Cold storage ratio
                </p>
              </CardContent>
            </Card>
          </div>

          {/* Reserve Ratio Chart */}
          <Card>
            <CardHeader>
              <CardTitle>Reserve Ratio History</CardTitle>
              <CardDescription>7-day reserve ratio trend</CardDescription>
            </CardHeader>
            <CardContent>
              <ResponsiveContainer width="100%" height={300}>
                <BarChart data={reserveHistory}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="date" />
                  <YAxis domain={['dataMin - 1', 'dataMax + 1']} />
                  <Tooltip />
                  <Bar dataKey="ratio" fill="hsl(var(--primary))" />
                </BarChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>

          {/* Alert Thresholds */}
          <Card>
            <CardHeader>
              <CardTitle>Alert Configuration</CardTitle>
              <CardDescription>Reserve ratio monitoring thresholds</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="flex items-center justify-between p-3 border rounded-lg">
                  <div className="flex items-center space-x-3">
                    <AlertTriangle className="w-5 h-5 text-red-500" />
                    <div>
                      <p className="font-medium">Critical Threshold</p>
                      <p className="text-sm text-muted-foreground">Immediate action required</p>
                    </div>
                  </div>
                  <Badge variant="destructive">100%</Badge>
                </div>
                <div className="flex items-center justify-between p-3 border rounded-lg">
                  <div className="flex items-center space-x-3">
                    <AlertTriangle className="w-5 h-5 text-yellow-500" />
                    <div>
                      <p className="font-medium">Warning Threshold</p>
                      <p className="text-sm text-muted-foreground">Monitor closely</p>
                    </div>
                  </div>
                  <Badge variant="outline">101%</Badge>
                </div>
                <div className="flex items-center justify-between p-3 border rounded-lg">
                  <div className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-green-500" />
                    <div>
                      <p className="font-medium">Optimal Range</p>
                      <p className="text-sm text-muted-foreground">Healthy operation</p>
                    </div>
                  </div>
                  <Badge variant="default">102-105%</Badge>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="holdings" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Holdings Breakdown */}
            <Card>
              <CardHeader>
                <CardTitle>Bitcoin Holdings Distribution</CardTitle>
                <CardDescription>Breakdown by wallet type</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  {bitcoinHoldings.map((holding, index) => (
                    <div key={index} className="flex items-center justify-between p-3 border rounded-lg">
                      <div>
                        <p className="font-medium">{holding.type}</p>
                        <p className="text-sm text-muted-foreground font-mono">{holding.address}</p>
                      </div>
                      <div className="text-right">
                        <p className="font-mono">{holding.amount.toFixed(8)} BTC</p>
                        <p className="text-sm text-muted-foreground">{holding.percentage}%</p>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            {/* Visual Distribution */}
            <Card>
              <CardHeader>
                <CardTitle>Holdings Visualization</CardTitle>
                <CardDescription>Distribution pie chart</CardDescription>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={250}>
                  <PieChart>
                    <Pie
                      data={pieData}
                      cx="50%"
                      cy="50%"
                      outerRadius={80}
                      dataKey="value"
                      label={({ name, value }) => `${name}: ${value}%`}
                    >
                      {pieData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={entry.color} />
                      ))}
                    </Pie>
                    <Tooltip />
                  </PieChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>
          </div>

          {/* Security Measures */}
          <Card>
            <CardHeader>
              <CardTitle>Security Measures</CardTitle>
              <CardDescription>Multi-signature and custody controls</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-3">
                  <div className="flex justify-between">
                    <span>Multi-sig Threshold</span>
                    <Badge>3 of 5</Badge>
                  </div>
                  <div className="flex justify-between">
                    <span>Cold Storage</span>
                    <Badge variant="default">Enabled</Badge>
                  </div>
                  <div className="flex justify-between">
                    <span>Time Locks</span>
                    <Badge variant="default">24h Delay</Badge>
                  </div>
                </div>
                <div className="space-y-3">
                  <div className="flex justify-between">
                    <span>Hardware Security</span>
                    <Badge variant="default">HSM Protected</Badge>
                  </div>
                  <div className="flex justify-between">
                    <span>Backup Locations</span>
                    <Badge>3 Sites</Badge>
                  </div>
                  <div className="flex justify-between">
                    <span>Last Audit</span>
                    <Badge variant="outline">Jan 2024</Badge>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="transactions" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Recent Bitcoin Transactions</CardTitle>
              <CardDescription>Deposits and withdrawals processed by the reserve manager</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {recentTransactions.map((tx) => (
                  <div key={tx.id} className="flex items-center justify-between p-3 border rounded-lg">
                    <div className="flex items-center space-x-3">
                      {getStatusIcon(tx.status)}
                      {getTransactionIcon(tx.type)}
                      <div>
                        <p className="font-medium capitalize">{tx.type}</p>
                        <p className="text-sm text-muted-foreground font-mono">{tx.txHash}</p>
                      </div>
                    </div>
                    <div className="text-right">
                      <p className="font-mono">{tx.amount.toFixed(8)} BTC</p>
                      <div className="flex items-center space-x-2 text-sm text-muted-foreground">
                        <span>{tx.confirmations}/6 confirmations</span>
                        <Badge variant="outline" className="text-xs">
                          {tx.status}
                        </Badge>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="proof" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Proof of Reserves</CardTitle>
              <CardDescription>Cryptographic proof of Bitcoin holdings</CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-4">
                  <h4 className="font-medium">Latest Proof</h4>
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span>Generated:</span>
                      <span>2024-01-15 12:00 UTC</span>
                    </div>
                    <div className="flex justify-between">
                      <span>Block Height:</span>
                      <span>825,000</span>
                    </div>
                    <div className="flex justify-between">
                      <span>Total BTC:</span>
                      <span className="font-mono">{totalBTC.toFixed(8)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span>Merkle Root:</span>
                      <span className="font-mono">a1b2c3d4...</span>
                    </div>
                  </div>
                </div>
                
                <div className="space-y-4">
                  <h4 className="font-medium">Verification</h4>
                  <div className="space-y-2">
                    <div className="flex items-center space-x-2">
                      <CheckCircle className="w-4 h-4 text-green-500" />
                      <span className="text-sm">Cryptographic signature valid</span>
                    </div>
                    <div className="flex items-center space-x-2">
                      <CheckCircle className="w-4 h-4 text-green-500" />
                      <span className="text-sm">Block confirmations verified</span>
                    </div>
                    <div className="flex items-center space-x-2">
                      <CheckCircle className="w-4 h-4 text-green-500" />
                      <span className="text-sm">Balance consistency confirmed</span>
                    </div>
                  </div>
                </div>
              </div>

              <div className="border-t pt-4">
                <Button 
                  onClick={handleGenerateProof}
                  disabled={isGeneratingProof}
                  className="flex items-center space-x-2"
                >
                  <RefreshCw className={`w-4 h-4 ${isGeneratingProof ? 'animate-spin' : ''}`} />
                  <span>
                    {isGeneratingProof ? 'Generating Proof...' : 'Generate New Proof'}
                  </span>
                </Button>
                <p className="text-sm text-muted-foreground mt-2">
                  Proof generation typically takes 2-3 minutes to complete
                </p>
              </div>

              <div className="bg-muted p-4 rounded-lg">
                <h4 className="font-medium mb-2">Proof Hash</h4>
                <p className="font-mono text-sm break-all">
                  0x7d4f8a9b2c5e3f6d8a1b4c7e9f2a5d8b6e3c9f1a4d7b5e8c2f6a9d3b7e1c4f8a2
                </p>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}