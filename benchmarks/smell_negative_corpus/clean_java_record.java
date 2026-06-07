// Guards against: Data Class FP.
// Java records are the idiomatic way to create immutable data carriers —
// they are explicitly designed to be "data classes" and are not a smell.

package negative;

import java.time.LocalDate;
import java.util.List;

/**
 * Immutable person record using Java 16+ record syntax.
 *
 * Records provide automatic equals, hashCode, toString, and accessor
 * methods. This is the intended design — DTOs should not contain
 * business logic.
 */
public record Person(
    String firstName,
    String lastName,
    int age,
    String email,
    String phone,
    String address
) {
    /** Compact constructor with validation. */
    public Person {
        if (firstName == null || firstName.isBlank()) {
            throw new IllegalArgumentException("firstName is required");
        }
        if (email == null || !email.contains("@")) {
            throw new IllegalArgumentException("valid email is required");
        }
        if (age < 0) {
            throw new IllegalArgumentException("age cannot be negative");
        }
    }

    /** Derived full name. */
    public String fullName() {
        return firstName + " " + lastName;
    }
}

/**
 * Event record for audit logging.
 */
public record AuditEntry(
    String action,
    String actor,
    String resource,
    LocalDate timestamp,
    List<String> details
) {
    public AuditEntry {
        details = List.copyOf(details); // defensive copy for immutability
        timestamp = timestamp != null ? timestamp : LocalDate.now();
    }
}

/**
 * Configuration record for service settings.
 */
public record ServiceConfig(
    String host,
    int port,
    int timeoutMs,
    int maxRetries,
    boolean tlsEnabled
) {
    /** Returns the base URL derived from config fields. */
    public String baseUrl() {
        String scheme = tlsEnabled ? "https" : "http";
        return scheme + "://" + host + ":" + port;
    }
}
