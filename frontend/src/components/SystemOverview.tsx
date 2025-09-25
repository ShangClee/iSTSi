import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/card';
import { Progress } from './ui/progress';
import { Badge } from './ui/badge';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, AreaChart, Area } from 'recharts';
import { TrendingUp, TrendingDown, Users, Shield, Database, Activity, Bitcoin, Coins } from 'lucide-react';

interface SystemOverviewProps {
  systemState: {
    status: string;
    totalOperations: number;
    successRate: number;
    reserveRatio: number;
    activeUsers: number;
    lastUpdate: string;
  };
}

// Mock performance data
const performanceData = [
  { time: '00:00', operations: 125, reserves: 102.1 },
  { time: '04:00', operations: 89, reserves: 102.3 },
  { time: '08:00', operations: 234, reserves: 102.0 },
  { time: '12:00', operations: 456, reserves: 101.8 },
  { time: '16:00', operations: 345, reserves: 102.2 },
  { time: '20:00', operations: 278, reserves: 102.4 },
];

const contractStats = [
  { name: 'Integration Router', status: 'operational', uptime: 99.9, gasUsed: 2456789 },
  { name: 'KYC Registry', status: 'operational', uptime: 99.8, gasUsed: 1234567 },
  { name: 'iSTSi Token', status: 'operational', uptime: 99.9, gasUsed: 3456789 },
  { name: 'Reserve Manager', status: 'operational', uptime: 99.7, gasUsed: 987654 },
];

export function SystemOverview({ systemState }: SystemOverviewProps) {
  const formatNumber = (num: number) => {
    return new Intl.NumberFormat().format(num);
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'operational': return <TrendingUp className="w-4 h-4 text-green-500" />;
      case 'degraded': return <TrendingDown className="w-4 h-4 text-yellow-500" />;
      default: return <TrendingDown className="w-4 h-4 text-red-500" />;
    }
  };

  return (
    <div className="space-y-6">
      {/* Key Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Operations</CardTitle>
            <Activity className="w-4 h-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{formatNumber(systemState.totalOperations)}</div>
            <p className="text-xs text-muted-foreground">
              +12.5% from last month
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
            <Shield className="w-4 h-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{systemState.successRate.toFixed(1)}%</div>
            <Progress value={systemState.successRate} className="mt-2" />
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Reserve Ratio</CardTitle>
            <Database className="w-4 h-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{systemState.reserveRatio.toFixed(1)}%</div>
            <p className="text-xs text-muted-foreground">
              Above minimum threshold
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Users</CardTitle>
            <Users className="w-4 h-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{formatNumber(systemState.activeUsers)}</div>
            <p className="text-xs text-muted-foreground">
              +4.2% from yesterday
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Performance Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle>Operations Over Time</CardTitle>
            <CardDescription>24-hour operation volume</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <AreaChart data={performanceData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" />
                <YAxis />
                <Tooltip />
                <Area type="monotone" dataKey="operations" stroke="hsl(var(--primary))" fill="hsl(var(--primary))" fillOpacity={0.2} />
              </AreaChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Reserve Ratio Trend</CardTitle>
            <CardDescription>Bitcoin reserves vs token supply</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={performanceData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" />
                <YAxis domain={['dataMin - 0.5', 'dataMax + 0.5']} />
                <Tooltip />
                <Line type="monotone" dataKey="reserves" stroke="hsl(var(--chart-2))" strokeWidth={2} />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      </div>

      {/* Contract Status */}
      <Card>
        <CardHeader>
          <CardTitle>Contract Status Overview</CardTitle>
          <CardDescription>Real-time status of all integrated contracts</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {contractStats.map((contract, index) => (
              <div key={index} className="flex items-center justify-between p-4 border rounded-lg">
                <div className="flex items-center space-x-4">
                  {getStatusIcon(contract.status)}
                  <div>
                    <p className="font-medium">{contract.name}</p>
                    <p className="text-sm text-muted-foreground">
                      Uptime: {contract.uptime}%
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <Badge variant="default">
                    {contract.status.toUpperCase()}
                  </Badge>
                  <p className="text-sm text-muted-foreground mt-1">
                    Gas: {formatNumber(contract.gasUsed)}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Token Statistics */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Bitcoin className="w-5 h-5" />
              <span>Bitcoin Reserves</span>
            </CardTitle>
            <CardDescription>Current Bitcoin holdings</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex justify-between">
                <span>Total BTC</span>
                <span className="font-mono">45.67891234 BTC</span>
              </div>
              <div className="flex justify-between">
                <span>USD Value</span>
                <span className="font-mono">$1,827,456.78</span>
              </div>
              <div className="flex justify-between">
                <span>Cold Storage</span>
                <span className="font-mono">95.2%</span>
              </div>
              <div className="flex justify-between">
                <span>Hot Wallet</span>
                <span className="font-mono">4.8%</span>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Coins className="w-5 h-5" />
              <span>Token Supply</span>
            </CardTitle>
            <CardDescription>iSTSi token metrics</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex justify-between">
                <span>Total Supply</span>
                <span className="font-mono">4,567,891,234 iSTSi</span>
              </div>
              <div className="flex justify-between">
                <span>Circulating</span>
                <span className="font-mono">4,467,891,234 iSTSi</span>
              </div>
              <div className="flex justify-between">
                <span>Market Cap</span>
                <span className="font-mono">$1,825,678.90</span>
              </div>
              <div className="flex justify-between">
                <span>Backing Ratio</span>
                <span className="font-mono text-green-600">102.3%</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}