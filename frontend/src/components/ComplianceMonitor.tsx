import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { Badge } from './ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { Progress } from './ui/progress';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, LineChart, Line } from 'recharts';
import { Shield, UserCheck, AlertTriangle, CheckCircle, XCircle, Search, Eye, Ban, TrendingUp } from 'lucide-react';

const kycStats = {
  totalUsers: 12547,
  verifiedUsers: 11892,
  pendingVerification: 428,
  rejectedApplications: 227,
  complianceRate: 94.8,
};

const complianceData = [
  { date: '2024-01-10', verified: 1150, pending: 45, rejected: 8 },
  { date: '2024-01-11', verified: 1230, pending: 52, rejected: 12 },
  { date: '2024-01-12', verified: 1180, pending: 38, rejected: 6 },
  { date: '2024-01-13', verified: 1340, pending: 65, rejected: 15 },
  { date: '2024-01-14', verified: 1280, pending: 41, rejected: 9 },
  { date: '2024-01-15', verified: 1420, pending: 58, rejected: 11 },
];

const riskScoreDistribution = [
  { range: '0-20', count: 8420, percentage: 67.1 },
  { range: '21-40', count: 2890, percentage: 23.0 },
  { range: '41-60', count: 945, percentage: 7.5 },
  { range: '61-80', count: 234, percentage: 1.9 },
  { range: '81-100', count: 58, percentage: 0.5 },
];

const pendingKYC = [
  {
    id: 'KYC-001',
    user: '0xuser1...1234',
    tier: 'Tier 2',
    documents: 'ID, Proof of Address',
    submittedAt: '2024-01-15 10:30:00',
    status: 'under_review',
    riskScore: 25,
  },
  {
    id: 'KYC-002',
    user: '0xuser2...5678',
    tier: 'Tier 3',
    documents: 'ID, Bank Statement, Source of Funds',
    submittedAt: '2024-01-15 09:15:00',
    status: 'additional_info_required',
    riskScore: 45,
  },
  {
    id: 'KYC-003',
    user: '0xuser3...9012',
    tier: 'Tier 1',
    documents: 'ID',
    submittedAt: '2024-01-15 08:45:00',
    status: 'ready_for_approval',
    riskScore: 15,
  },
];

const blacklistedAddresses = [
  {
    address: '0xbad1...1111',
    reason: 'Sanctions list',
    addedDate: '2024-01-10',
    addedBy: 'Compliance Officer',
    severity: 'high',
  },
  {
    address: '0xbad2...2222',
    reason: 'Suspicious activity',
    addedDate: '2024-01-12',
    addedBy: 'Automated System',
    severity: 'medium',
  },
];

const complianceEvents = [
  {
    id: 'CE-001',
    type: 'kyc_approved',
    user: '0xuser4...4567',
    details: 'Tier 2 KYC approved',
    timestamp: '2024-01-15 14:30:00',
    officer: 'John Doe',
  },
  {
    id: 'CE-002',
    type: 'address_blacklisted',
    user: '0xbad3...3333',
    details: 'Added to sanctions list',
    timestamp: '2024-01-15 14:15:00',
    officer: 'Jane Smith',
  },
  {
    id: 'CE-003',
    type: 'transaction_blocked',
    user: '0xuser5...7890',
    details: 'Exceeded daily limit',
    timestamp: '2024-01-15 14:00:00',
    officer: 'Automated System',
  },
];

export function ComplianceMonitor() {
  const [selectedAddress, setSelectedAddress] = useState('');
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const [isSearching, setIsSearching] = useState(false);

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'under_review': return <Eye className="w-4 h-4 text-blue-500" />;
      case 'additional_info_required': return <AlertTriangle className="w-4 h-4 text-yellow-500" />;
      case 'ready_for_approval': return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'rejected': return <XCircle className="w-4 h-4 text-red-500" />;
      default: return <Eye className="w-4 h-4 text-gray-500" />;
    }
  };

  const getRiskBadge = (score: number) => {
    if (score <= 20) return <Badge variant="default" className="bg-green-100 text-green-800">Low</Badge>;
    if (score <= 40) return <Badge variant="default" className="bg-yellow-100 text-yellow-800">Medium</Badge>;
    if (score <= 60) return <Badge variant="default" className="bg-orange-100 text-orange-800">High</Badge>;
    return <Badge variant="destructive">Critical</Badge>;
  };

  const getSeverityBadge = (severity: string) => {
    switch (severity) {
      case 'low': return <Badge variant="outline">Low</Badge>;
      case 'medium': return <Badge variant="default" className="bg-yellow-100 text-yellow-800">Medium</Badge>;
      case 'high': return <Badge variant="destructive">High</Badge>;
      default: return <Badge variant="outline">Unknown</Badge>;
    }
  };

  const handleAddressSearch = async () => {
    if (!selectedAddress) return;
    
    setIsSearching(true);
    // Simulate search
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    setSearchResults([
      {
        address: selectedAddress,
        kycStatus: 'verified',
        tier: 'Tier 2',
        riskScore: 25,
        lastActivity: '2024-01-15 12:30:00',
        totalTransactions: 45,
        volume: '125.5 BTC',
      }
    ]);
    setIsSearching(false);
  };

  return (
    <div className="space-y-6">
      <Tabs defaultValue="overview" className="space-y-4">
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="kyc">KYC Management</TabsTrigger>
          <TabsTrigger value="blacklist">Blacklist</TabsTrigger>
          <TabsTrigger value="events">Compliance Events</TabsTrigger>
          <TabsTrigger value="search">Address Lookup</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="space-y-4">
          {/* KYC Statistics */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Total Users</CardTitle>
                <UserCheck className="w-4 h-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{kycStats.totalUsers.toLocaleString()}</div>
                <p className="text-xs text-muted-foreground">
                  +5.2% from last month
                </p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Verified Users</CardTitle>
                <CheckCircle className="w-4 h-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{kycStats.verifiedUsers.toLocaleString()}</div>
                <Progress value={(kycStats.verifiedUsers / kycStats.totalUsers) * 100} className="mt-2" />
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Pending Review</CardTitle>
                <Eye className="w-4 h-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{kycStats.pendingVerification}</div>
                <p className="text-xs text-muted-foreground">
                  Avg processing: 24h
                </p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Compliance Rate</CardTitle>
                <Shield className="w-4 h-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{kycStats.complianceRate}%</div>
                <p className="text-xs text-muted-foreground">
                  Industry leading
                </p>
              </CardContent>
            </Card>
          </div>

          {/* Compliance Trends */}
          <Card>
            <CardHeader>
              <CardTitle>KYC Processing Trends</CardTitle>
              <CardDescription>Daily KYC verification statistics</CardDescription>
            </CardHeader>
            <CardContent>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={complianceData}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="date" />
                  <YAxis />
                  <Tooltip />
                  <Line type="monotone" dataKey="verified" strokeWidth={2} stroke="hsl(var(--chart-1))" />
                  <Line type="monotone" dataKey="pending" strokeWidth={2} stroke="hsl(var(--chart-2))" />
                  <Line type="monotone" dataKey="rejected" strokeWidth={2} stroke="hsl(var(--chart-3))" />
                </LineChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>

          {/* Risk Score Distribution */}
          <Card>
            <CardHeader>
              <CardTitle>Risk Score Distribution</CardTitle>
              <CardDescription>User risk assessment breakdown</CardDescription>
            </CardHeader>
            <CardContent>
              <ResponsiveContainer width="100%" height={250}>
                <BarChart data={riskScoreDistribution}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="range" />
                  <YAxis />
                  <Tooltip />
                  <Bar dataKey="count" fill="hsl(var(--primary))" />
                </BarChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="kyc" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Pending KYC Applications</CardTitle>
              <CardDescription>Applications requiring manual review</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {pendingKYC.map((application) => (
                  <div key={application.id} className="p-4 border rounded-lg">
                    <div className="flex items-center justify-between mb-3">
                      <div className="flex items-center space-x-3">
                        {getStatusIcon(application.status)}
                        <div>
                          <p className="font-medium">{application.id}</p>
                          <p className="text-sm text-muted-foreground">{application.user}</p>
                        </div>
                      </div>
                      <div className="flex items-center space-x-2">
                        {getRiskBadge(application.riskScore)}
                        <Badge variant="outline">{application.tier}</Badge>
                      </div>
                    </div>
                    
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                      <div>
                        <span className="font-medium">Documents:</span>
                        <p className="text-muted-foreground">{application.documents}</p>
                      </div>
                      <div>
                        <span className="font-medium">Submitted:</span>
                        <p className="text-muted-foreground">{application.submittedAt}</p>
                      </div>
                      <div>
                        <span className="font-medium">Risk Score:</span>
                        <p className="text-muted-foreground">{application.riskScore}/100</p>
                      </div>
                    </div>
                    
                    <div className="flex space-x-2 mt-4">
                      <Button size="sm" variant="default">Approve</Button>
                      <Button size="sm" variant="outline">Request Info</Button>
                      <Button size="sm" variant="destructive">Reject</Button>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="blacklist" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Blacklisted Addresses</CardTitle>
              <CardDescription>Addresses blocked from system interaction</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="flex space-x-2">
                  <Input placeholder="Enter address to blacklist..." className="flex-1" />
                  <Select>
                    <SelectTrigger className="w-48">
                      <SelectValue placeholder="Reason" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="sanctions">Sanctions list</SelectItem>
                      <SelectItem value="suspicious">Suspicious activity</SelectItem>
                      <SelectItem value="fraud">Fraud investigation</SelectItem>
                      <SelectItem value="other">Other</SelectItem>
                    </SelectContent>
                  </Select>
                  <Button className="flex items-center space-x-2">
                    <Ban className="w-4 h-4" />
                    <span>Add</span>
                  </Button>
                </div>

                <div className="space-y-3">
                  {blacklistedAddresses.map((address, index) => (
                    <div key={index} className="flex items-center justify-between p-3 border rounded-lg">
                      <div className="flex items-center space-x-3">
                        <Ban className="w-5 h-5 text-red-500" />
                        <div>
                          <p className="font-mono text-sm">{address.address}</p>
                          <p className="text-sm text-muted-foreground">{address.reason}</p>
                        </div>
                      </div>
                      <div className="flex items-center space-x-2">
                        {getSeverityBadge(address.severity)}
                        <Button size="sm" variant="outline">Remove</Button>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="events" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Recent Compliance Events</CardTitle>
              <CardDescription>Audit trail of compliance actions</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {complianceEvents.map((event) => (
                  <div key={event.id} className="flex items-center justify-between p-3 border rounded-lg">
                    <div className="flex items-center space-x-3">
                      <div className="w-2 h-2 bg-blue-500 rounded-full"></div>
                      <div>
                        <p className="font-medium">{event.type.replace(/_/g, ' ').toUpperCase()}</p>
                        <p className="text-sm text-muted-foreground">{event.details}</p>
                      </div>
                    </div>
                    <div className="text-right text-sm text-muted-foreground">
                      <p>{event.timestamp}</p>
                      <p>by {event.officer}</p>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="search" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Address Compliance Lookup</CardTitle>
              <CardDescription>Search for KYC status and compliance information</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex space-x-2">
                <Input 
                  placeholder="Enter wallet address (0x...)" 
                  value={selectedAddress}
                  onChange={(e) => setSelectedAddress(e.target.value)}
                  className="flex-1"
                />
                <Button 
                  onClick={handleAddressSearch}
                  disabled={!selectedAddress || isSearching}
                  className="flex items-center space-x-2"
                >
                  <Search className="w-4 h-4" />
                  <span>{isSearching ? 'Searching...' : 'Search'}</span>
                </Button>
              </div>

              {searchResults.length > 0 && (
                <div className="space-y-4">
                  {searchResults.map((result, index) => (
                    <div key={index} className="p-4 border rounded-lg bg-muted/50">
                      <h4 className="font-medium mb-3">Compliance Information</h4>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div className="space-y-2">
                          <div className="flex justify-between">
                            <span>Address:</span>
                            <span className="font-mono text-sm">{result.address}</span>
                          </div>
                          <div className="flex justify-between">
                            <span>KYC Status:</span>
                            <Badge variant="default">{result.kycStatus}</Badge>
                          </div>
                          <div className="flex justify-between">
                            <span>KYC Tier:</span>
                            <Badge variant="outline">{result.tier}</Badge>
                          </div>
                          <div className="flex justify-between">
                            <span>Risk Score:</span>
                            {getRiskBadge(result.riskScore)}
                          </div>
                        </div>
                        <div className="space-y-2">
                          <div className="flex justify-between">
                            <span>Last Activity:</span>
                            <span className="text-sm">{result.lastActivity}</span>
                          </div>
                          <div className="flex justify-between">
                            <span>Total Transactions:</span>
                            <span>{result.totalTransactions}</span>
                          </div>
                          <div className="flex justify-between">
                            <span>Total Volume:</span>
                            <span className="font-mono">{result.volume}</span>
                          </div>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}