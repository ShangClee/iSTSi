import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { Badge } from './ui/badge';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { Switch } from './ui/switch';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';
import { AlertTriangle, Bell, CheckCircle, XCircle, Clock, Shield, TrendingDown, Database, Users, Settings } from 'lucide-react';

interface Alert {
  id: string;
  type: 'critical' | 'warning' | 'info';
  category: 'security' | 'compliance' | 'operational' | 'system';
  title: string;
  message: string;
  timestamp: string;
  acknowledged: boolean;
  resolved: boolean;
  source: string;
  affectedComponent?: string;
  actionRequired?: boolean;
}

interface AlertRule {
  id: string;
  name: string;
  category: string;
  condition: string;
  threshold: string;
  enabled: boolean;
  recipients: string[];
  severity: 'critical' | 'warning' | 'info';
}

const mockAlerts: Alert[] = [
  {
    id: 'ALT-001',
    type: 'warning',
    category: 'operational',
    title: 'Reserve Ratio Approaching Minimum',
    message: 'Bitcoin reserve ratio has dropped to 101.2%, approaching the minimum threshold of 100%',
    timestamp: '2024-01-15 14:45:00',
    acknowledged: false,
    resolved: false,
    source: 'Reserve Manager',
    affectedComponent: 'Reserve Validation',
    actionRequired: true
  },
  {
    id: 'ALT-002',
    type: 'info',
    category: 'compliance',
    title: 'Daily Compliance Report Generated',
    message: 'Daily KYC compliance report has been generated and is ready for review',
    timestamp: '2024-01-15 12:00:00',
    acknowledged: true,
    resolved: false,
    source: 'KYC Registry',
    actionRequired: false
  },
  {
    id: 'ALT-003',
    type: 'critical',
    category: 'security',
    title: 'Multiple Failed Authentication Attempts',
    message: 'User 0xuser...1234 has exceeded failed authentication attempts. Account has been temporarily locked',
    timestamp: '2024-01-15 11:30:00',
    acknowledged: true,
    resolved: true,
    source: 'Authentication System',
    affectedComponent: 'User Authentication',
    actionRequired: false
  },
  {
    id: 'ALT-004',
    type: 'warning',
    category: 'system',
    title: 'High Gas Usage Detected',
    message: 'Integration Router operations are consuming 25% more gas than normal baseline',
    timestamp: '2024-01-15 10:15:00',
    acknowledged: false,
    resolved: false,
    source: 'Integration Router',
    affectedComponent: 'Gas Optimization',
    actionRequired: true
  },
  {
    id: 'ALT-005',
    type: 'info',
    category: 'operational',
    title: 'Scheduled Maintenance Completed',
    message: 'Routine system maintenance has been completed successfully',
    timestamp: '2024-01-15 08:00:00',
    acknowledged: true,
    resolved: true,
    source: 'System Administrator',
    actionRequired: false
  }
];

const alertRules: AlertRule[] = [
  {
    id: 'RULE-001',
    name: 'Reserve Ratio Critical',
    category: 'operational',
    condition: 'reserve_ratio < threshold',
    threshold: '100%',
    enabled: true,
    recipients: ['admin@company.com', 'operations@company.com'],
    severity: 'critical'
  },
  {
    id: 'RULE-002',
    name: 'Reserve Ratio Warning',
    category: 'operational',
    condition: 'reserve_ratio < threshold',
    threshold: '102%',
    enabled: true,
    recipients: ['operations@company.com'],
    severity: 'warning'
  },
  {
    id: 'RULE-003',
    name: 'Failed Authentication Attempts',
    category: 'security',
    condition: 'failed_attempts > threshold',
    threshold: '5',
    enabled: true,
    recipients: ['security@company.com'],
    severity: 'critical'
  },
  {
    id: 'RULE-004',
    name: 'High Gas Usage',
    category: 'system',
    condition: 'gas_usage > baseline * threshold',
    threshold: '1.5',
    enabled: true,
    recipients: ['tech@company.com'],
    severity: 'warning'
  }
];

interface AlertCenterProps {
  alerts: Alert[];
}

export function AlertCenter({ alerts: propAlerts }: AlertCenterProps) {
  const [alerts, setAlerts] = useState<Alert[]>(mockAlerts);
  const [rules, setRules] = useState<AlertRule[]>(alertRules);
  const [filter, setFilter] = useState('all');
  const [categoryFilter, setCategoryFilter] = useState('all');

  const getAlertIcon = (type: string, resolved: boolean) => {
    if (resolved) return <CheckCircle className="w-4 h-4 text-green-500" />;
    
    switch (type) {
      case 'critical': return <XCircle className="w-4 h-4 text-red-500" />;
      case 'warning': return <AlertTriangle className="w-4 h-4 text-yellow-500" />;
      case 'info': return <Bell className="w-4 h-4 text-blue-500" />;
      default: return <Clock className="w-4 h-4 text-gray-500" />;
    }
  };

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'security': return <Shield className="w-4 h-4" />;
      case 'compliance': return <CheckCircle className="w-4 h-4" />;
      case 'operational': return <TrendingDown className="w-4 h-4" />;
      case 'system': return <Database className="w-4 h-4" />;
      default: return <Bell className="w-4 h-4" />;
    }
  };

  const getAlertBadge = (type: string, resolved: boolean) => {
    if (resolved) return <Badge variant="outline" className="bg-green-50 text-green-700">Resolved</Badge>;
    
    switch (type) {
      case 'critical': return <Badge variant="destructive">Critical</Badge>;
      case 'warning': return <Badge variant="default" className="bg-yellow-100 text-yellow-800">Warning</Badge>;
      case 'info': return <Badge variant="default" className="bg-blue-100 text-blue-800">Info</Badge>;
      default: return <Badge variant="outline">Unknown</Badge>;
    }
  };

  const getSeverityBadge = (severity: string) => {
    switch (severity) {
      case 'critical': return <Badge variant="destructive">Critical</Badge>;
      case 'warning': return <Badge variant="default" className="bg-yellow-100 text-yellow-800">Warning</Badge>;
      case 'info': return <Badge variant="default" className="bg-blue-100 text-blue-800">Info</Badge>;
      default: return <Badge variant="outline">Unknown</Badge>;
    }
  };

  const handleAcknowledge = (alertId: string) => {
    setAlerts(prev => prev.map(alert => 
      alert.id === alertId ? { ...alert, acknowledged: true } : alert
    ));
  };

  const handleResolve = (alertId: string) => {
    setAlerts(prev => prev.map(alert => 
      alert.id === alertId ? { ...alert, resolved: true, acknowledged: true } : alert
    ));
  };

  const toggleRule = (ruleId: string) => {
    setRules(prev => prev.map(rule => 
      rule.id === ruleId ? { ...rule, enabled: !rule.enabled } : rule
    ));
  };

  const filteredAlerts = alerts.filter(alert => {
    if (filter === 'unresolved' && alert.resolved) return false;
    if (filter === 'unacknowledged' && alert.acknowledged) return false;
    if (categoryFilter !== 'all' && alert.category !== categoryFilter) return false;
    return true;
  });

  const alertStats = {
    total: alerts.length,
    critical: alerts.filter(a => a.type === 'critical' && !a.resolved).length,
    warning: alerts.filter(a => a.type === 'warning' && !a.resolved).length,
    unacknowledged: alerts.filter(a => !a.acknowledged).length,
    resolved: alerts.filter(a => a.resolved).length
  };

  return (
    <div className="space-y-6">
      <Tabs defaultValue="alerts" className="space-y-4">
        <TabsList>
          <TabsTrigger value="alerts">Active Alerts</TabsTrigger>
          <TabsTrigger value="rules">Alert Rules</TabsTrigger>
          <TabsTrigger value="config">Configuration</TabsTrigger>
        </TabsList>

        <TabsContent value="alerts" className="space-y-4">
          {/* Alert Statistics */}
          <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Total Alerts</CardTitle>
                <Bell className="w-4 h-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{alertStats.total}</div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Critical</CardTitle>
                <XCircle className="w-4 h-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-red-600">{alertStats.critical}</div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Warnings</CardTitle>
                <AlertTriangle className="w-4 h-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-yellow-600">{alertStats.warning}</div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Unacknowledged</CardTitle>
                <Clock className="w-4 h-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{alertStats.unacknowledged}</div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Resolved</CardTitle>
                <CheckCircle className="w-4 h-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-green-600">{alertStats.resolved}</div>
              </CardContent>
            </Card>
          </div>

          {/* Filters */}
          <Card>
            <CardHeader>
              <CardTitle>Filter Alerts</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex space-x-4">
                <Select value={filter} onValueChange={setFilter}>
                  <SelectTrigger className="w-48">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All Alerts</SelectItem>
                    <SelectItem value="unresolved">Unresolved</SelectItem>
                    <SelectItem value="unacknowledged">Unacknowledged</SelectItem>
                  </SelectContent>
                </Select>

                <Select value={categoryFilter} onValueChange={setCategoryFilter}>
                  <SelectTrigger className="w-48">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All Categories</SelectItem>
                    <SelectItem value="security">Security</SelectItem>
                    <SelectItem value="compliance">Compliance</SelectItem>
                    <SelectItem value="operational">Operational</SelectItem>
                    <SelectItem value="system">System</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardContent>
          </Card>

          {/* Alerts List */}
          <Card>
            <CardHeader>
              <CardTitle>Alert Feed</CardTitle>
              <CardDescription>System alerts and notifications</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {filteredAlerts.map((alert) => (
                  <div key={alert.id} className={`p-4 border rounded-lg ${alert.acknowledged ? 'bg-muted/30' : 'bg-background'}`}>
                    <div className="flex items-start justify-between mb-3">
                      <div className="flex items-center space-x-3">
                        {getAlertIcon(alert.type, alert.resolved)}
                        {getCategoryIcon(alert.category)}
                        <div>
                          <h4 className="font-medium">{alert.title}</h4>
                          <p className="text-sm text-muted-foreground">{alert.message}</p>
                        </div>
                      </div>
                      <div className="flex items-center space-x-2">
                        {getAlertBadge(alert.type, alert.resolved)}
                        {alert.actionRequired && (
                          <Badge variant="outline" className="bg-orange-50 text-orange-700">
                            Action Required
                          </Badge>
                        )}
                      </div>
                    </div>

                    <div className="flex items-center justify-between text-sm text-muted-foreground">
                      <div className="flex items-center space-x-4">
                        <span>Source: {alert.source}</span>
                        <span>Time: {alert.timestamp}</span>
                        {alert.affectedComponent && (
                          <span>Component: {alert.affectedComponent}</span>
                        )}
                      </div>
                      
                      <div className="flex space-x-2">
                        {!alert.acknowledged && (
                          <Button 
                            size="sm" 
                            variant="outline"
                            onClick={() => handleAcknowledge(alert.id)}
                          >
                            Acknowledge
                          </Button>
                        )}
                        {!alert.resolved && (
                          <Button 
                            size="sm" 
                            variant="default"
                            onClick={() => handleResolve(alert.id)}
                          >
                            Resolve
                          </Button>
                        )}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="rules" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Alert Rules Configuration</CardTitle>
              <CardDescription>Manage automated alert rules and thresholds</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {rules.map((rule) => (
                  <div key={rule.id} className="p-4 border rounded-lg">
                    <div className="flex items-center justify-between mb-3">
                      <div className="flex items-center space-x-3">
                        <Switch 
                          checked={rule.enabled}
                          onCheckedChange={() => toggleRule(rule.id)}
                        />
                        <div>
                          <h4 className="font-medium">{rule.name}</h4>
                          <p className="text-sm text-muted-foreground">{rule.condition}</p>
                        </div>
                      </div>
                      <div className="flex items-center space-x-2">
                        {getSeverityBadge(rule.severity)}
                        <Badge variant="outline">{rule.category}</Badge>
                      </div>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                      <div>
                        <span className="font-medium">Threshold:</span>
                        <p className="text-muted-foreground">{rule.threshold}</p>
                      </div>
                      <div>
                        <span className="font-medium">Recipients:</span>
                        <p className="text-muted-foreground">{rule.recipients.length} contacts</p>
                      </div>
                      <div className="flex space-x-2">
                        <Button size="sm" variant="outline">Edit</Button>
                        <Button size="sm" variant="outline">Test</Button>
                      </div>
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
              <CardTitle>Notification Settings</CardTitle>
              <CardDescription>Configure how and when alerts are delivered</CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-4">
                  <h4 className="font-medium">Email Notifications</h4>
                  <div className="space-y-3">
                    <div className="flex items-center justify-between">
                      <Label htmlFor="email-critical">Critical Alerts</Label>
                      <Switch id="email-critical" defaultChecked />
                    </div>
                    <div className="flex items-center justify-between">
                      <Label htmlFor="email-warning">Warning Alerts</Label>
                      <Switch id="email-warning" defaultChecked />
                    </div>
                    <div className="flex items-center justify-between">
                      <Label htmlFor="email-info">Info Alerts</Label>
                      <Switch id="email-info" />
                    </div>
                  </div>
                </div>

                <div className="space-y-4">
                  <h4 className="font-medium">SMS Notifications</h4>
                  <div className="space-y-3">
                    <div className="flex items-center justify-between">
                      <Label htmlFor="sms-critical">Critical Alerts</Label>
                      <Switch id="sms-critical" defaultChecked />
                    </div>
                    <div className="flex items-center justify-between">
                      <Label htmlFor="sms-warning">Warning Alerts</Label>
                      <Switch id="sms-warning" />
                    </div>
                    <div className="flex items-center justify-between">
                      <Label htmlFor="sms-info">Info Alerts</Label>
                      <Switch id="sms-info" />
                    </div>
                  </div>
                </div>
              </div>

              <div className="space-y-4">
                <h4 className="font-medium">Contact Information</h4>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="admin-email">Admin Email</Label>
                    <Input id="admin-email" type="email" placeholder="admin@company.com" />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="operations-email">Operations Email</Label>
                    <Input id="operations-email" type="email" placeholder="ops@company.com" />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="security-email">Security Email</Label>
                    <Input id="security-email" type="email" placeholder="security@company.com" />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="emergency-phone">Emergency Phone</Label>
                    <Input id="emergency-phone" type="tel" placeholder="+1-555-0123" />
                  </div>
                </div>
              </div>

              <div className="flex space-x-4">
                <Button>Save Configuration</Button>
                <Button variant="outline">Test Notifications</Button>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}