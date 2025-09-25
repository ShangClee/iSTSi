import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Badge } from './ui/badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from './ui/table';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from './ui/dialog';
import { CheckCircle, XCircle, Clock, AlertTriangle, Search, Filter, Download, Eye, Bitcoin, Coins, ArrowRightLeft, RefreshCw } from 'lucide-react';

interface Operation {
  id: string;
  type: 'BitcoinDeposit' | 'TokenWithdrawal' | 'CrossTokenExchange' | 'ComplianceCheck';
  user: string;
  amount: string;
  status: 'completed' | 'pending' | 'failed' | 'cancelled';
  timestamp: string;
  gasUsed: number;
  txHash?: string;
  errorMessage?: string;
  compliance: {
    kycVerified: boolean;
    riskScore: number;
    tier: string;
  };
  details: any;
}

const mockOperations: Operation[] = [
  {
    id: 'OP-2024-001234',
    type: 'BitcoinDeposit',
    user: '0xuser1...ab12',
    amount: '1.25000000 BTC',
    status: 'completed',
    timestamp: '2024-01-15 14:32:15',
    gasUsed: 125000,
    txHash: '0x7d2b3f4a...',
    compliance: { kycVerified: true, riskScore: 15, tier: 'Tier 2' },
    details: {
      btcTxHash: 'd4f8a9b2c5e3f6d8a1b4c7e9f2a5d8b6',
      confirmations: 6,
      processingTime: '4m 32s'
    }
  },
  {
    id: 'OP-2024-001235',
    type: 'TokenWithdrawal',
    user: '0xuser2...cd34',
    amount: '75000 iSTSi',
    status: 'pending',
    timestamp: '2024-01-15 14:28:45',
    gasUsed: 98000,
    compliance: { kycVerified: true, riskScore: 25, tier: 'Tier 2' },
    details: {
      btcAddress: 'bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
      estimatedCompletion: '8 minutes',
      currentStep: 'Reserve Validation'
    }
  },
  {
    id: 'OP-2024-001236',
    type: 'CrossTokenExchange',
    user: '0xuser3...ef56',
    amount: '10000 FT → 25000 iSTSi',
    status: 'failed',
    timestamp: '2024-01-15 14:25:30',
    gasUsed: 156000,
    errorMessage: 'Insufficient reserve ratio for exchange',
    compliance: { kycVerified: true, riskScore: 35, tier: 'Tier 1' },
    details: {
      fromToken: 'Fungible Token',
      toToken: 'iSTSi Token',
      exchangeRate: '2.5:1',
      failureReason: 'Reserve threshold breach'
    }
  },
  {
    id: 'OP-2024-001237',
    type: 'ComplianceCheck',
    user: '0xuser4...gh78',
    amount: 'KYC Verification',
    status: 'completed',
    timestamp: '2024-01-15 14:20:12',
    gasUsed: 45000,
    compliance: { kycVerified: true, riskScore: 12, tier: 'Tier 3' },
    details: {
      checkType: 'Tier Upgrade',
      documentsVerified: ['ID', 'Proof of Address', 'Bank Statement'],
      processingTime: '2h 15m'
    }
  },
  {
    id: 'OP-2024-001238',
    type: 'BitcoinDeposit',
    user: '0xuser5...ij90',
    amount: '0.50000000 BTC',
    status: 'cancelled',
    timestamp: '2024-01-15 14:15:08',
    gasUsed: 65000,
    errorMessage: 'User cancelled during KYC verification',
    compliance: { kycVerified: false, riskScore: 85, tier: 'Tier 0' },
    details: {
      btcTxHash: 'pending',
      cancellationReason: 'User request',
      refundProcessed: true
    }
  }
];

export function OperationsLog() {
  const [operations, setOperations] = useState<Operation[]>(mockOperations);
  const [filteredOps, setFilteredOps] = useState<Operation[]>(mockOperations);
  const [selectedOperation, setSelectedOperation] = useState<Operation | null>(null);
  const [filters, setFilters] = useState({
    status: 'all',
    type: 'all',
    user: '',
    dateFrom: '',
    dateTo: ''
  });

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed': return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'pending': return <Clock className="w-4 h-4 text-yellow-500" />;
      case 'failed': return <XCircle className="w-4 h-4 text-red-500" />;
      case 'cancelled': return <AlertTriangle className="w-4 h-4 text-gray-500" />;
      default: return <Clock className="w-4 h-4 text-gray-500" />;
    }
  };

  const getOperationIcon = (type: string) => {
    switch (type) {
      case 'BitcoinDeposit': return <Bitcoin className="w-4 h-4 text-orange-500" />;
      case 'TokenWithdrawal': return <Coins className="w-4 h-4 text-blue-500" />;
      case 'CrossTokenExchange': return <ArrowRightLeft className="w-4 h-4 text-purple-500" />;
      case 'ComplianceCheck': return <CheckCircle className="w-4 h-4 text-green-500" />;
      default: return <RefreshCw className="w-4 h-4 text-gray-500" />;
    }
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'completed': return <Badge variant="default" className="bg-green-100 text-green-800">Completed</Badge>;
      case 'pending': return <Badge variant="default" className="bg-yellow-100 text-yellow-800">Pending</Badge>;
      case 'failed': return <Badge variant="destructive">Failed</Badge>;
      case 'cancelled': return <Badge variant="outline">Cancelled</Badge>;
      default: return <Badge variant="outline">Unknown</Badge>;
    }
  };

  const getRiskBadge = (score: number) => {
    if (score <= 20) return <Badge variant="default" className="bg-green-100 text-green-800">Low</Badge>;
    if (score <= 40) return <Badge variant="default" className="bg-yellow-100 text-yellow-800">Medium</Badge>;
    if (score <= 60) return <Badge variant="default" className="bg-orange-100 text-orange-800">High</Badge>;
    return <Badge variant="destructive">Critical</Badge>;
  };

  const handleFilter = () => {
    let filtered = operations;

    if (filters.status !== 'all') {
      filtered = filtered.filter(op => op.status === filters.status);
    }

    if (filters.type !== 'all') {
      filtered = filtered.filter(op => op.type === filters.type);
    }

    if (filters.user) {
      filtered = filtered.filter(op => 
        op.user.toLowerCase().includes(filters.user.toLowerCase())
      );
    }

    setFilteredOps(filtered);
  };

  const clearFilters = () => {
    setFilters({
      status: 'all',
      type: 'all',
      user: '',
      dateFrom: '',
      dateTo: ''
    });
    setFilteredOps(operations);
  };

  const exportOperations = () => {
    // Simulate CSV export
    const csv = [
      ['ID', 'Type', 'User', 'Amount', 'Status', 'Timestamp', 'Gas Used'].join(','),
      ...filteredOps.map(op => [
        op.id,
        op.type,
        op.user,
        op.amount,
        op.status,
        op.timestamp,
        op.gasUsed
      ].join(','))
    ].join('\n');

    const blob = new Blob([csv], { type: 'text/csv' });
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'operations-export.csv';
    a.click();
  };

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Operations Filter</CardTitle>
          <CardDescription>Filter and search through system operations</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
            <Select value={filters.status} onValueChange={(value) => setFilters({...filters, status: value})}>
              <SelectTrigger>
                <SelectValue placeholder="Status" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Status</SelectItem>
                <SelectItem value="completed">Completed</SelectItem>
                <SelectItem value="pending">Pending</SelectItem>
                <SelectItem value="failed">Failed</SelectItem>
                <SelectItem value="cancelled">Cancelled</SelectItem>
              </SelectContent>
            </Select>

            <Select value={filters.type} onValueChange={(value) => setFilters({...filters, type: value})}>
              <SelectTrigger>
                <SelectValue placeholder="Type" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Types</SelectItem>
                <SelectItem value="BitcoinDeposit">Bitcoin Deposit</SelectItem>
                <SelectItem value="TokenWithdrawal">Token Withdrawal</SelectItem>
                <SelectItem value="CrossTokenExchange">Cross Token Exchange</SelectItem>
                <SelectItem value="ComplianceCheck">Compliance Check</SelectItem>
              </SelectContent>
            </Select>

            <Input 
              placeholder="User address..."
              value={filters.user}
              onChange={(e) => setFilters({...filters, user: e.target.value})}
            />

            <Button onClick={handleFilter} className="flex items-center space-x-2">
              <Filter className="w-4 h-4" />
              <span>Filter</span>
            </Button>

            <Button variant="outline" onClick={clearFilters}>
              Clear
            </Button>
          </div>

          <div className="flex justify-between items-center mt-4">
            <p className="text-sm text-muted-foreground">
              Showing {filteredOps.length} of {operations.length} operations
            </p>
            <Button variant="outline" onClick={exportOperations} className="flex items-center space-x-2">
              <Download className="w-4 h-4" />
              <span>Export CSV</span>
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Recent Operations</CardTitle>
          <CardDescription>Detailed log of all system operations</CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Operation</TableHead>
                <TableHead>User</TableHead>
                <TableHead>Amount</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Risk</TableHead>
                <TableHead>Timestamp</TableHead>
                <TableHead>Gas</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredOps.map((operation) => (
                <TableRow key={operation.id}>
                  <TableCell>
                    <div className="flex items-center space-x-2">
                      {getOperationIcon(operation.type)}
                      <div>
                        <p className="font-medium text-sm">{operation.type}</p>
                        <p className="text-xs text-muted-foreground">{operation.id}</p>
                      </div>
                    </div>
                  </TableCell>
                  <TableCell>
                    <div>
                      <p className="font-mono text-sm">{operation.user}</p>
                      <p className="text-xs text-muted-foreground">{operation.compliance.tier}</p>
                    </div>
                  </TableCell>
                  <TableCell className="font-mono text-sm">{operation.amount}</TableCell>
                  <TableCell>
                    <div className="flex items-center space-x-2">
                      {getStatusIcon(operation.status)}
                      {getStatusBadge(operation.status)}
                    </div>
                  </TableCell>
                  <TableCell>
                    {getRiskBadge(operation.compliance.riskScore)}
                  </TableCell>
                  <TableCell className="text-sm">{operation.timestamp}</TableCell>
                  <TableCell className="text-sm">{operation.gasUsed.toLocaleString()}</TableCell>
                  <TableCell>
                    <Dialog>
                      <DialogTrigger asChild>
                        <Button 
                          variant="outline" 
                          size="sm"
                          onClick={() => setSelectedOperation(operation)}
                        >
                          <Eye className="w-4 h-4" />
                        </Button>
                      </DialogTrigger>
                      <DialogContent className="max-w-2xl">
                        <DialogHeader>
                          <DialogTitle>Operation Details</DialogTitle>
                          <DialogDescription>
                            Detailed information for operation {selectedOperation?.id}
                          </DialogDescription>
                        </DialogHeader>
                        {selectedOperation && (
                          <div className="space-y-6">
                            {/* Basic Info */}
                            <div className="grid grid-cols-2 gap-4">
                              <div>
                                <h4 className="font-medium mb-2">Basic Information</h4>
                                <div className="space-y-2 text-sm">
                                  <div className="flex justify-between">
                                    <span>ID:</span>
                                    <span className="font-mono">{selectedOperation.id}</span>
                                  </div>
                                  <div className="flex justify-between">
                                    <span>Type:</span>
                                    <span>{selectedOperation.type}</span>
                                  </div>
                                  <div className="flex justify-between">
                                    <span>User:</span>
                                    <span className="font-mono">{selectedOperation.user}</span>
                                  </div>
                                  <div className="flex justify-between">
                                    <span>Amount:</span>
                                    <span className="font-mono">{selectedOperation.amount}</span>
                                  </div>
                                  <div className="flex justify-between">
                                    <span>Status:</span>
                                    {getStatusBadge(selectedOperation.status)}
                                  </div>
                                </div>
                              </div>

                              <div>
                                <h4 className="font-medium mb-2">Compliance</h4>
                                <div className="space-y-2 text-sm">
                                  <div className="flex justify-between">
                                    <span>KYC Verified:</span>
                                    <Badge variant={selectedOperation.compliance.kycVerified ? "default" : "destructive"}>
                                      {selectedOperation.compliance.kycVerified ? 'Yes' : 'No'}
                                    </Badge>
                                  </div>
                                  <div className="flex justify-between">
                                    <span>Risk Score:</span>
                                    {getRiskBadge(selectedOperation.compliance.riskScore)}
                                  </div>
                                  <div className="flex justify-between">
                                    <span>KYC Tier:</span>
                                    <Badge variant="outline">{selectedOperation.compliance.tier}</Badge>
                                  </div>
                                </div>
                              </div>
                            </div>

                            {/* Technical Details */}
                            <div>
                              <h4 className="font-medium mb-2">Technical Details</h4>
                              <div className="grid grid-cols-2 gap-4 text-sm">
                                <div className="flex justify-between">
                                  <span>Timestamp:</span>
                                  <span>{selectedOperation.timestamp}</span>
                                </div>
                                <div className="flex justify-between">
                                  <span>Gas Used:</span>
                                  <span>{selectedOperation.gasUsed.toLocaleString()}</span>
                                </div>
                                {selectedOperation.txHash && (
                                  <div className="flex justify-between">
                                    <span>TX Hash:</span>
                                    <span className="font-mono">{selectedOperation.txHash}</span>
                                  </div>
                                )}
                              </div>
                            </div>

                            {/* Operation Specific Details */}
                            <div>
                              <h4 className="font-medium mb-2">Operation Details</h4>
                              <div className="bg-muted p-3 rounded-lg">
                                <pre className="text-sm">
                                  {JSON.stringify(selectedOperation.details, null, 2)}
                                </pre>
                              </div>
                            </div>

                            {/* Error Message */}
                            {selectedOperation.errorMessage && (
                              <div>
                                <h4 className="font-medium mb-2 text-red-600">Error Details</h4>
                                <div className="bg-red-50 border border-red-200 p-3 rounded-lg">
                                  <p className="text-sm text-red-800">{selectedOperation.errorMessage}</p>
                                </div>
                              </div>
                            )}
                          </div>
                        )}
                      </DialogContent>
                    </Dialog>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  );
}