# Bitcoin Custody Integration System - Design Guidelines

## Loco.rs-Inspired Architecture Guidelines

This system follows Loco.rs design principles to ensure compatibility with Rust web frameworks and facilitate seamless Figma-to-Rust workflows.

### General Guidelines

* Follow MVC (Model-View-Controller) architectural patterns in component structure
* Use semantic naming conventions that align with Rust development practices
* Prioritize responsive layouts using flexbox and grid over absolute positioning
* Structure components hierarchically: controls → models → views → tasks
* Maintain clean, modular code with single-responsibility components
* Ensure all interactive elements are properly accessible and keyboard navigable

### Component Naming Conventions

* **Container Components**: Use descriptive names like `SystemOverlay`, `DashboardHud`, `AlertCenter`
* **Layout Components**: Follow Rust-style naming: `HeaderFrame`, `ContentFrame`, `SidebarFrame`
* **Control Components**: Name based on function: `EmergencyPauseControl`, `StatusIndicator`, `MetricDisplay`
* **Data Components**: Use model-based naming: `ReserveData`, `ComplianceModel`, `OperationTask`

### Layout Structure Guidelines

* **Root Containers**: Always use semantic container elements (`<main>`, `<header>`, `<section>`)
* **Responsive Design**: Use CSS Grid for main layouts, Flexbox for component internals
* **Constraint-Based Layout**: Define explicit horizontal/vertical constraints for responsive behavior
* **Auto Layout**: Implement consistent spacing using CSS custom properties and utility classes

### Icon and Asset Guidelines

* **Vector Icons**: Use Lucide React icons consistently throughout the system
* **Custom Icons**: Flatten to single vectors or export as optimized PNG/SVG
* **Asset Organization**: Group related icons and maintain consistent sizing (16px, 20px, 24px)
* **Accessibility**: Ensure all icons have proper ARIA labels and semantic meaning

## Design System Specifications

### Typography Hierarchy

* **Display Text**: Use existing CSS custom properties (--text-2xl, --text-xl)
* **Body Text**: Default to --text-base with --font-weight-normal
* **Labels**: Use --text-base with --font-weight-medium
* **Captions**: Use --text-sm for supplementary information
* **Monospace**: Apply font-mono class for addresses, hashes, and technical data

### Color Semantic Usage

* **Primary Actions**: Use `--primary` for main interactive elements
* **Destructive Actions**: Use `--destructive` for emergency/critical operations
* **Status Indicators**: 
  - Operational: `--chart-1` (green)
  - Warning: `--chart-4` (yellow)
  - Critical: `--destructive` (red)
  - Info: `--chart-2` (blue)
* **Background Hierarchy**: Use `--background`, `--card`, `--muted` for layered content

### Component-Specific Guidelines

#### Dashboard Cards
* Use `Card` component with consistent padding and spacing
* Include proper `CardHeader` with title and description
* Maintain visual hierarchy with appropriate content grouping
* Implement responsive grid layouts for metric displays

#### Data Tables
* Use `Table` component with proper semantic structure
* Include sortable headers where appropriate
* Implement proper loading and empty states
* Ensure mobile responsiveness with horizontal scroll

#### Forms and Controls
* Use consistent spacing between form elements
* Implement proper validation states and error messaging
* Group related controls using fieldset semantic structure
* Maintain consistent button grouping and hierarchy

#### Status and Alerts
* Use consistent icon and color patterns for status indicators
* Implement progressive disclosure for detailed information
* Ensure critical alerts are prominently displayed
* Use Badge components for categorical information

#### Navigation and Tabs
* Maintain consistent active/inactive states
* Use proper ARIA labels and keyboard navigation
* Implement responsive behavior for mobile viewports
* Group related functionality logically

### Accessibility Requirements

* **Keyboard Navigation**: All interactive elements must be keyboard accessible
* **Screen Readers**: Use proper ARIA labels and semantic HTML
* **Color Contrast**: Ensure minimum 4.5:1 contrast ratios for text
* **Focus Management**: Implement visible focus indicators
* **Error Handling**: Provide clear, accessible error messages

### Performance Guidelines

* **Bundle Size**: Keep individual components under 50KB
* **Lazy Loading**: Implement code splitting for large components
* **Data Fetching**: Use efficient polling intervals (5s for critical data)
* **Rendering**: Minimize re-renders using React.memo and useCallback where appropriate

### Testing and Quality Assurance

* **Component Testing**: Each component should have corresponding tests
* **Integration Testing**: Test cross-component interactions
* **Accessibility Testing**: Verify WCAG 2.1 AA compliance
* **Performance Testing**: Monitor bundle sizes and render performance

### Future Rust/Loco.rs Migration Considerations

* **State Management**: Structure state to be serializable for Rust integration
* **API Interfaces**: Design data models compatible with Rust serialization
* **Component Boundaries**: Maintain clear separation of concerns for easier migration
* **Type Safety**: Use TypeScript strictly to facilitate Rust type system alignment

## Figma Design Workflow Guidelines

### Figma Structure Requirements

* **Use Frames Only**: Always use Frames (not Groups) for containers to ensure proper export
* **Naming Convention**: Name root Frames as 'Overlay' or 'Hud' to match Rust UI requirements
* **Icon Preparation**: Flatten all icons to vectors or export as single PNG/SVG files
* **Constraints Configuration**: Set Left/Right/Center/Scale constraints in Figma, then prototype to test scaling
* **Layer Consistency**: Maintain identical layer structures for repeated components to ensure clean export

### Design System Integration

* **Auto Layout Usage**: Use Auto Layout for responsive elements that mirror web UI behavior
* **Component Variants**: Create consistent component variants that map to React props
* **Token Management**: Use design tokens that directly correlate to CSS custom properties
* **Spacing System**: Follow 4px, 8px, 16px, 24px, 32px spacing increments
* **Color System**: Use semantic color naming that matches CSS custom property structure

### Export Optimization

* **Asset Preparation**: Optimize SVGs and ensure proper sizing before export
* **Component Hierarchy**: Structure components to match React component tree
* **Responsive Behavior**: Test all breakpoints and constraint behaviors
* **Plugin Compatibility**: Ensure designs are compatible with Figma-to-Rust export plugins