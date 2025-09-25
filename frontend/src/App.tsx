import React, { useState, useEffect } from 'react';
import { SystemOverview } from '@/components/SystemOverview';
import { IntegrationRouter } from '@/components/IntegrationRouter';
import { ReserveManager } from '@/components/ReserveManager';
import { ComplianceMonitor } from '@/components/ComplianceMonitor';
import { OperationsLog } from '@/components/OperationsLog';
import { AlertCenter } from '@/components/AlertCenter';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Bell, Shield, Activity, Database, Settings, Pause, Play } from 'lucide-react';

// Mock data for demonstration
const mockSystemState = {
  status: 'operational', // operational | degraded | emergency | paused
  totalOperations: 45672,
  successRate: 99.7,
  reserveRatio: 102.3,
  activeUsers: 1248,
  lastUpdate: new Date().toISOString(),
};

const mockAlerts = [
  {
    id: '1',
    type: 'warning',
    message: 'Reserve ratio approaching minimum threshold',
    timestamp: new Date(Date.now() - 300000).toISOString(),
    severity: 'medium'
  },
  {
    id: '2',
    type: 'info',
    message: 'Daily compliance report generated',
    timestamp: new Date(Date.now() - 1800000).toISOString(),
    severity: 'low'
  }
];

export default function App() {
  const [systemState, setSystemState] = useState(mockSystemState);
  const [alerts, setAlerts] = useState(mockAlerts);
  const [activeTab, setActiveTab] = useState('overview');

  // Simulate real-time updates
  useEffect(() => {
    const interval = setInterval(() => {
      setSystemState(prev => ({
        ...prev,
        totalOperations: prev.totalOperations + Math.floor(Math.random() * 5),
        successRate: 99.5 + Math.random() * 0.5,
        reserveRatio: 101 + Math.random() * 2,
        activeUsers: prev.activeUsers + Math.floor(Math.random() * 10 - 5),
        lastUpdate: new Date().toISOString(),
      }));
    }, 5000);

    return () => clearInterval(interval);
  }, []);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'operational': return 'bg-green-500';
      case 'degraded': return 'bg-yellow-500';
      case 'emergency': return 'bg-red-500';
      case 'paused': return 'bg-gray-500';
      default: return 'bg-gray-500';
    }
  };

  const handleEmergencyPause = () => {
    setSystemState(prev => ({
      ...prev,
      status: prev.status === 'paused' ? 'operational' : 'paused'
    }));
  };

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b bg-card">
        <div className="container mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <Shield className="w-8 h-8 text-primary" />
              <div>
                <h1 className="text-2xl font-semibold">Bitcoin Custody Integration System</h1>
                <p className="text-sm text-muted-foreground">Multi-Contract Management Dashboard</p>
              </div>
            </div>
            
            <div className="flex items-center space-x-4">
              {/* System Status */}
              <div className="flex items-center space-x-2">
                <div className={`w-3 h-3 rounded-full ${getStatusColor(systemState.status)}`} />
                <Badge variant={systemState.status === 'operational' ? 'default' : 'destructive'}>
                  {systemState.status.toUpperCase()}
                </Badge>
              </div>
              
              {/* Alerts */}
              <Button variant="ghost" size="sm" className="relative">
                <Bell className="w-4 h-4" />
                {alerts.length > 0 && (
                  <span className="absolute -top-1 -right-1 w-5 h-5 bg-red-500 text-white rounded-full text-xs flex items-center justify-center">
                    {alerts.length}
                  </span>
                )}
              </Button>
              
              {/* Emergency Controls */}
              <Button
                variant={systemState.status === 'paused' ? 'default' : 'destructive'}
                size="sm"
                onClick={handleEmergencyPause}
                className="flex items-center space-x-2"
              >
                {systemState.status === 'paused' ? (
                  <>
                    <Play className="w-4 h-4" />
                    <span>Resume</span>
                  </>
                ) : (
                  <>
                    <Pause className="w-4 h-4" />
                    <span>Emergency Pause</span>
                  </>
                )}
              </Button>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="container mx-auto px-6 py-6">
        <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
          <TabsList className="grid grid-cols-6 w-full max-w-4xl">
            <TabsTrigger value="overview" className="flex items-center space-x-2">
              <Activity className="w-4 h-4" />
              <span>Overview</span>
            </TabsTrigger>
            <TabsTrigger value="router" className="flex items-center space-x-2">
              <Settings className="w-4 h-4" />
              <span>Router</span>
            </TabsTrigger>
            <TabsTrigger value="reserves" className="flex items-center space-x-2">
              <Database className="w-4 h-4" />
              <span>Reserves</span>
            </TabsTrigger>
            <TabsTrigger value="compliance" className="flex items-center space-x-2">
              <Shield className="w-4 h-4" />
              <span>Compliance</span>
            </TabsTrigger>
            <TabsTrigger value="operations" className="flex items-center space-x-2">
              <Activity className="w-4 h-4" />
              <span>Operations</span>
            </TabsTrigger>
            <TabsTrigger value="alerts" className="flex items-center space-x-2">
              <Bell className="w-4 h-4" />
              <span>Alerts</span>
            </TabsTrigger>
          </TabsList>

          <TabsContent value="overview">
            <SystemOverview systemState={systemState} />
          </TabsContent>

          <TabsContent value="router">
            <IntegrationRouter />
          </TabsContent>

          <TabsContent value="reserves">
            <ReserveManager systemState={systemState} />
          </TabsContent>

          <TabsContent value="compliance">
            <ComplianceMonitor />
          </TabsContent>

          <TabsContent value="operations">
            <OperationsLog />
          </TabsContent>

          <TabsContent value="alerts">
            <AlertCenter alerts={alerts} />
          </TabsContent>
        </Tabs>
      </main>
    </div>
  );
}