// Guards against: Switch Statements FP.
// Java enum switch with expression syntax is exhaustive and type-safe —
// the compiler enforces completeness, making this a safe dispatch pattern.

package negative;

/**
 * HTTP status codes relevant to error categorization.
 */
public enum HttpStatus {
    OK(200),
    CREATED(201),
    NO_CONTENT(204),
    BAD_REQUEST(400),
    UNAUTHORIZED(401),
    NOT_FOUND(404),
    CONFLICT(409),
    INTERNAL_ERROR(500),
    BAD_GATEWAY(502),
    SERVICE_UNAVAILABLE(503);

    private final int code;

    HttpStatus(int code) {
        this.code = code;
    }

    /** Return the numeric HTTP status code. */
    public int code() {
        return code;
    }

    /**
     * Categorize a status into a human-readable group.
     *
     * The switch expression is exhaustive — adding a new enum constant
     * without a case causes a compile error. This is not a switch smell.
     */
    public String getCategory() {
        return switch (this) {
            case OK, CREATED, NO_CONTENT -> "success";
            case BAD_REQUEST, UNAUTHORIZED, NOT_FOUND, CONFLICT -> "client error";
            case INTERNAL_ERROR, BAD_GATEWAY, SERVICE_UNAVAILABLE -> "server error";
        };
    }

    /**
     * Return true if this status represents a successful response.
     */
    public boolean isSuccess() {
        return code >= 200 && code < 300;
    }

    /**
     * Return true if this status represents a client error.
     */
    public boolean isClientError() {
        return code >= 400 && code < 500;
    }

    /**
     * Return true if this status represents a server error.
     */
    public boolean isServerError() {
        return code >= 500;
    }

    /**
     * Resolve an HTTP status code to the enum value.
     *
     * @throws IllegalArgumentException if the code does not match any status.
     */
    public static HttpStatus fromCode(int code) {
        for (HttpStatus status : values()) {
            if (status.code == code) {
                return status;
            }
        }
        throw new IllegalArgumentException("unknown HTTP status: " + code);
    }
}
